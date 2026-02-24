// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use dictation_ia_lib::CliArgs;

fn main() {
    let cli_args = CliArgs::parse();
    dictation_ia_lib::run(cli_args)
}
