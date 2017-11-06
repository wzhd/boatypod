#[derive(StructOpt, Debug)]
#[structopt(name = "boatypod", about = "A podcast downloader for newsboat.")]
pub struct Opt {
    #[structopt(short = "p", long = "progress", help = "Show progress bar.")]
    pub progress: bool,

    #[structopt(short = "n", long = "num", help = "Limit number of items to download.", default_value = "42")]
    pub num: usize,
}
