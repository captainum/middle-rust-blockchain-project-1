//! Библиотека, реализующая Derive-макрос ReadWrite, имплементирующий тела методов
//! [`read_from`], [`write_from`] структур для взаимодействия с заданными форматами данных.
//!
//! Формат данных определяется при помощи атрибута [`format`], который может принимать
//! следующие значения:
//! 1. text - текстовый формат описания списка операций;
//! 2. csv - таблица банковских операций;
//! 3. bin - бинарное представление списка операций.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(ReadWrite, attributes(format))]
pub fn read_write_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut fmt = "text";

    for attr in &input.attrs {
        if attr.path().is_ident("format") {
            if let Ok(meta) = attr.parse_args::<syn::LitStr>() {
                let val = meta.value();
                match val.as_str() {
                    "text" => fmt = "text",
                    "csv" => fmt = "csv",
                    "bin" => fmt = "bin",
                    _ => panic!("Unknown format"),
                }
            }
        }
    }

    let read_from_body = match fmt {
        "text" => quote! {
            while !reader.fill_buf()?.is_empty() {
                records.push(Record::from_text(&mut reader)?);
            }
        },
        "csv" => quote! {
            let mut header = String::new();
            reader.read_line(&mut header)?;

            if header.ends_with('\n') {
                header.truncate(header.len() - 1);
            }

            Self::validate_header(&header)?;

            loop {
                if reader.fill_buf()?.is_empty() {
                    break;
                }

                records.push(Record::from_csv(&mut reader)?);
            }
        },
        "bin" => quote! {
            while !reader.fill_buf()?.is_empty() {
                records.push(Record::from_bin(&mut reader)?);
            }
        },
        _ => panic!("Unknown input format"),
    };

    let write_to_body = match fmt {
        "text" => quote! {
            for (i, record) in self.records.iter().enumerate() {
                if i > 0 {
                    writer.write_all(b"\n")?;
                }
                record.to_text(&mut writer)?;
            }
        },
        "csv" => quote! {
            let header = Self::prepare_header();
            writer
                .write_all(header.as_bytes())
                .map_err(|e| WriteError::WriteHeaderError(e.to_string()))?;
            writer.write_all(b"\n")?;

            for record in &self.records {
                record.to_csv(&mut writer)?;
            }
        },
        "bin" => quote! {
            for record in &self.records {
                record.to_bin(&mut writer)?;
            }
        },
        _ => panic!("Unknown output format"),
    };

    let expanded = quote! {
        impl YPBank for #name {
            fn read_from<R: Read>(r: &mut R) -> Result<Self, ReadError> {
                let mut reader = BufReader::new(r);

                let mut records: Vec<Record> = vec![];

                #read_from_body

                Ok(Self { records })
            }

            fn write_to<W: Write>(&self, w: &mut W) -> Result<(), WriteError> {
                let mut writer = BufWriter::new(w);

                #write_to_body

                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
