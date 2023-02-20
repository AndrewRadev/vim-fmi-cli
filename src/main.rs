use clap::{Parser, Subcommand};

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

    match args.command {
        Commands::Setup => println!("setup"),
        Commands::Put { task_id } => {
            println!("{}", task_id)
        },
        Commands::Version => {
            println!(::clap::crate_version!());
        },
    }

    // Continued program logic goes here...
}
