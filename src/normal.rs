use crate::opt::Opt;
use crate::{log_error, BANNER, HOST};
use async_std::task;
use chrono::Local;
use failure::{format_err, Error};
use read_input::prelude::*;
use std::process;

pub fn run(opts: Opt) {
    task::block_on(async {
        if let Err(e) = process(opts).await {
            log_error(&e);
            process::exit(1);
        };
    });

    if cfg!(target_os = "windows") {
        pause();
    }
}

async fn process(opts: Opt) -> Result<(), Error> {
    println!("{}", BANNER);

    let client = stats_api::Client::new();

    let date = if opts.date.is_some() {
        opts.date.unwrap()
    } else {
        Local::today().naive_local()
    };
    let todays_schedule = client.get_schedule_for(date).await?;

    println!("\nPick a game for {}...\n", date.format("%Y-%m-%d"));
    for (idx, game) in todays_schedule.games.iter().enumerate() {
        println!(
            "{}) {} - {} @ {}",
            idx + 1,
            game.date
                .with_timezone(&Local)
                .time()
                .format("%-I:%M %p")
                .to_string(),
            game.teams.away.detail.name,
            game.teams.home.detail.name
        );
    }

    let game_count = todays_schedule.games.len();
    let game_choice = input::<usize>()
        .msg("\n>>> ")
        .add_test(move |input| *input > 0 && *input <= game_count)
        .get();
    let game = todays_schedule.games[..]
        .get(game_choice - 1)
        .ok_or_else(|| format_err!("Invalid game choice"))?;

    let game_content = client.get_game_content(game.game_pk).await?;

    for epg in game_content.media.epg {
        if epg.title == "NHLTV" {
            if let Some(items) = epg.items {
                println!("\nPick a stream...\n");

                for (idx, stream) in items.iter().enumerate() {
                    println!("{}) {}", idx + 1, stream.media_feed_type);
                }

                let stream_count = items.len();
                let stream_choice = input::<usize>()
                    .msg("\n>>> ")
                    .add_test(move |input| *input > 0 && *input <= stream_count)
                    .get();
                let stream = items[..]
                    .get(stream_choice - 1)
                    .ok_or_else(|| format_err!("Invalid stream choice"))?;

                let url = format!(
                    "{}/getM3U8.php?league=nhl&date={}&id={}&cdn=akc",
                    HOST,
                    todays_schedule.date.format("%Y-%m-%d"),
                    stream.media_playback_id
                );

                println!("\n{}", url);
            }
        }
    }

    Ok(())
}

// Keep console window open until button press
fn pause() {
    use std::io::{self, prelude::*};

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "\nPress enter or close window to exit...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}
