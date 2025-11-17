// I felt like being cute?

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        eprintln!("{}", format!($($arg)*).blue());
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("{}", format!($($arg)*).yellow());
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("{}", format!($($arg)*).red());
    };
}
