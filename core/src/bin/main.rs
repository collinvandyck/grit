use clap::Parser;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    grit::bootstrap::install_hooks()?;
    let _opts = grit::opts::Opts::parse();
    tui()?;
    Ok(())
}

fn tui() -> Result<(), color_eyre::Report> {
    let opts = grit::opts::Opts::parse();
    let mut terminal = grit::bootstrap::init(&opts)?;
    grit::app::App::new(&opts)?.run(&mut terminal)?;
    grit::bootstrap::restore()?;
    Ok(())
}
