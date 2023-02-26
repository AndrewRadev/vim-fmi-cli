use clap::{Parser, Subcommand};
use url::Url;

use vim_fmi_cli::{Controller, Vim};

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
            // let output_path = controller.create(&task.output).unwrap();
            let vim = Vim::new(input_path);

            let contents = vim.run().unwrap();

            if contents.trim() == task.output {
                println!("Okay!");
            } else {
                println!("Wrong!");
            }
        },
        Commands::Version => {
            println!(::clap::crate_version!());
        },
    }

    // Continued program logic goes here...
}
