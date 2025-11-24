# CLI Comparer

Консольное приложение, использующее функциональность парсеров из крейта Parser.

Читает данные о транзакциях из двух файлов в указанных форматах и сравнивает их. В случае несовпадения сообщает,
какая транзакция не совпала.

Доступен help при указании флага --help

```
Usage:
    comparer --file1 [FILE] --format1 [FORMAT] --file2 [FILE] --format2 [FORMAT]

Options:
    --file1             First file to read
    --format1           Data format in the first file to read
    --file2             Second file to read
    --format2           Data format in the second file to read
    --help              Print this message
```
