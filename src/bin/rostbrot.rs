extern crate clap;
extern crate rostbrot;

use rostbrot::cache::{Cache, Configuration};
use rostbrot::color::colorize;

use clap::{App, Arg};
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = App::new("Rostbrot")
        .version("0.1.0")
        .author("Matthias Wolf <m@sushinara.net>")
        .about("Generate Buddhabrot images")
        .arg(
            Arg::with_name("cache")
                .takes_value(true)
                .long("cache")
                .help("A cache file to use; default: value of 'config' with extension 'cache'"),
        )
        .arg(
            Arg::with_name("config")
                .takes_value(true)
                .required(true)
                .index(1)
                .help("A yaml configuration file"),
        )
        .arg(
            Arg::with_name("filename")
                .takes_value(true)
                .required(true)
                .index(2)
                .help("The output filename"),
        )
        .get_matches();

    let config_filename = cli.value_of("config").unwrap();
    let config_filestub = Path::new(config_filename).file_stem().unwrap();
    let config = Configuration::load(config_filename).unwrap();

    let cache_filename_default = format!("{}.cache", config_filestub.to_str().unwrap());
    let cache_filename = cli.value_of("self").unwrap_or(&cache_filename_default);
    let mut cache = Cache::load(&cache_filename, &config);

    if !cache.valid {
        cache.populate();
        cache.dump(cache_filename).unwrap();
    }

    colorize(&cache, &config, cli.value_of("filename").unwrap())
}
