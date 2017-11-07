#[derive(StructOpt, Debug)]
#[structopt(name = "boatypod", about = "A podcast downloader for newsboat.")]
pub struct Opt {
    /// Show progress bar.
    #[structopt(short = "p", long = "progress")]
    pub progress: bool,

    /// Limit number of items to download.
    #[structopt(short = "n", long = "num", default_value = "42")]
    pub num: usize,
}
