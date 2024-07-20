use clap::Parser;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    grit::errors::install_hooks()?;
    let _opts = grit::opts::Opts::parse();
    tui()?;
    Ok(())
}

fn tui() -> Result<(), color_eyre::Report> {
    let opts = grit::opts::Opts::parse();
    let mut terminal = grit::tui::init(&opts)?;
    grit::app::App::new(&opts)?.run(&mut terminal)?;
    grit::tui::restore()?;
    Ok(())
}
