use clap::{Parser, Subcommand};
use url::Url;

use vim_fmi_cli::{Controller, Vim, Keylog};

#[derive(Debug, Parser)]
#[command(name = "vim-fmi")]
#[command(about = "Клиент за курса по Vim във ФМИ", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Инициализира потребителя, който ще праща решения
    Setup,

    /// Стартира упражнение с подадения идентификатор
    #[command(arg_required_else_help = true)]
    Put {
        /// Идентификатора на дадено упражнение
        task_id: String,
    },

    /// Показва текущата версия на клиента
    Version,
}

fn main() {
    let args = Cli::parse();
    let host = Url::parse("http://localhost:3000").unwrap();

    match args.command {
        Commands::Setup => println!("setup"),
        Commands::Put { task_id } => {
            let controller = Controller::new(host.clone(), &task_id).unwrap();
            let task = controller.download().unwrap();

            let input_path = controller.create_file("input", &task.input).unwrap();
            let log_path = controller.create_file("log", "").unwrap();
            let vimrc_path = controller.vimrc_path();
            let vim = Vim::new(input_path, log_path, vimrc_path);

            let (output, log_bytes) = vim.run().unwrap();
            let keylog = Keylog::new(&log_bytes);
            let script: String = keylog.into_iter().collect();

            if output.trim() == task.output {
                println!("Okay! Your keys were: {}", script);
                controller.upload(log_bytes).unwrap();
            } else {
                println!("Wrong! {}", script);
            }
        },
        Commands::Version => {
            println!(::clap::crate_version!());
        },
    }

    // Continued program logic goes here...
}
