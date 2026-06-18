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
        print_ln!($crate::cli::styles::success("✓"), $fmt $(, $($arg)*)?);
    };
}

/// Print to stderr with an error prefix [ERROR]
macro_rules! error_ln {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        eprint_ln!($crate::cli::styles::error("ERROR"), $fmt $(, $($arg)*)?);
    };
}

/// Print to stdout with a failed prefix [-]
macro_rules! failed_ln {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        print_ln!($crate::cli::styles::error("-"), $fmt $(, $($arg)*)?);
    };
}

/// Print to stdout with an info prefix [*]
macro_rules! info_ln {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        print_ln!($crate::cli::styles::info("*"), $fmt $(, $($arg)*)?);
    };
}

/// Print to stdout with a warning prefix [!]
macro_rules! warn_ln {
    ($fmt:literal $(, $($arg:tt)*)?) => {
        print_ln!($crate::cli::styles::warn("!"), $fmt $(, $($arg)*)?);
    };
}

/// Format into a String with a leading stylized prefix (like format!)
macro_rules! format_ln {
    ($prefix:expr, $fmt:literal $(, $($arg:tt)*)?) => {
        format!(concat!("[{}] ", $fmt), $prefix $(, $($arg)*)?)
    };
}
