# Проектная работа модуля 1. Чтение, парсинг и анализ данных в Rust

Для проекта используется [cargo workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) для удобства
общей сборки трех необходимых крейтов.

# Сборка проекта

```
cargo build
```

# Тестирование проекта

Запуск тестов

```
cargo test
```

Генерация HTML-отчета о покрытии:

```
cargo llvm-cov --html
```
