use lettre::{SendmailTransport, Transport};
use lettre_email::Email;
use rss::Channel;
use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use structopt::StructOpt;
use things_rss::{Article, Site};

const MAX_MAILS: usize = 100;

const SELECT_SQL: &str = "SELECT url from articles where url = ?;";
const INSERT_SQL: &str = "INSERT INTO articles (site, url) VALUES (?, ?);";

#[derive(StructOpt, Debug)]
#[structopt(name = "things-rss")]
struct Opt {
    #[structopt(short, long)]
    quiet: bool,

    #[structopt(long)]
    to: String,

    #[structopt(long)]
    from: String,
}

fn get_articles(url: &str) -> Vec<Article> {
    let channel = Channel::from_url(url).unwrap();
    channel
        .items()
        .iter()
        .filter_map(|item| {
            if let (Some(title), Some(link)) = (item.title(), item.link()) {
                Some(Article {
                    title: title.into(),
                    url: link.into(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn send_email(
    site: &Site,
    article: &Article,
    recipient: (&str, &str),
    sender: &str,
) -> Result<(), lettre::sendmail::error::Error> {
    let email = Email::builder()
        .to(recipient)
        .from(sender)
        .subject(format!("{}: {}", site.title, article.title))
        .text(&article.url)
        .build()
        .unwrap();

    // Open a local connection on port 25
    let mut mailer = SendmailTransport::new();

    mailer.send(email.into())
}

fn main() {
    let opt = Opt::from_args();

    let mut sent_mails = 0;

    let sites: Vec<Site> = things_rss::read_config("feeds.toml").unwrap();

    let conn = Connection::open("things-rss.sqlite3").unwrap();

    conn.execute(
        "create table if not exists articles (
             id integer primary key,
             site text not null,
             url text not null unique
         )",
        NO_PARAMS,
    )
    .unwrap();

    let mut select_stmt = conn.prepare(SELECT_SQL).unwrap();
    let mut insert_stmt = conn.prepare(INSERT_SQL).unwrap();

    for site in sites {
        if !opt.quiet {
            eprintln!("Checking {}...", site.title);
        }
        for ref article in get_articles(&site.url) {
            if sent_mails >= MAX_MAILS {
                std::process::exit(0);
            }

            if !select_stmt.exists(&[&article.url]).unwrap() {
                if !opt.quiet {
                    eprintln!("{}: {}", site.title, article.title);
                }

                match send_email(&site, &article, (&opt.to, ""), &opt.from) {
                    Ok(_) => sent_mails += 1,
                    Err(e) => {
                        eprintln!("Could not send email: {}", e);
                        std::process::exit(-1);
                    }
                }

                let _ = insert_stmt.execute(&[&site.title, &article.url]).unwrap();
            }
        }
    }
}
