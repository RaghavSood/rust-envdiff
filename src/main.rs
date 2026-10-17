use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let code = dotenv_diff::cli::run(
        std::env::args().skip(1),
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    );
    ExitCode::from(code as u8)
}
