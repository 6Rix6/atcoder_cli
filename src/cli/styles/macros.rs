macro_rules! print_ln {
    ($prefix:expr, $fmt:literal $(, $($arg:tt)*)?) => {
        println!(concat!("[{}] ", $fmt), $prefix $(, $($arg)*)?);
    };
}

macro_rules! eprint_ln {
    ($prefix:expr, $fmt:literal $(, $($arg:tt)*)?) => {
        eprintln!(concat!("[{}] ", $fmt), $prefix $(, $($arg)*)?);
    };
}

/// Print to stdout with a success prefix [✓]
macro_rules! success_ln {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        print_ln!($crate::cli::styles::success($crate::cli::styles::SUCCESS_PREFIX), $fmt $(, $($arg)*)?);
    };
}

/// Print to stderr with an error prefix [ERROR]
macro_rules! error_ln {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        eprint_ln!($crate::cli::styles::error($crate::cli::styles::ERROR_PREFIX), $fmt $(, $($arg)*)?);
    };
}

/// Print to stdout with a failed prefix [-]
macro_rules! failed_ln {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        print_ln!($crate::cli::styles::error($crate::cli::styles::FAILED_PREFIX), $fmt $(, $($arg)*)?);
    };
}

/// Print to stdout with an info prefix [*]
macro_rules! info_ln {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        print_ln!($crate::cli::styles::info($crate::cli::styles::INFO_PREFIX), $fmt $(, $($arg)*)?);
    };
}

/// Print to stdout with a warning prefix [!]
macro_rules! warn_ln {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        print_ln!($crate::cli::styles::warn($crate::cli::styles::WARN_PREFIX), $fmt $(, $($arg)*)?);
    };
}

/// Format into a String with a leading stylized prefix (like format!)
macro_rules! format_ln {
    ($prefix:expr, $fmt:literal $(, $($arg:tt)*)?) => {
        format!(concat!("[{}] ", $fmt), $prefix $(, $($arg)*)?)
    };
}
