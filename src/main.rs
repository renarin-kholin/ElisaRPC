use discord_rpc_client::Client;
use regex::Regex;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc;
use std::thread;
use std::{
    io::{Error, ErrorKind},
    process::{Command, Stdio},
};

fn main() -> Result<(), Error> {
    let mut drpc = Client::new(1230850847345348669);
    drpc.on_ready(|_ctx| {
        println!("ready...");
    });
    drpc.start();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let stdout1 = Command::new("playerctl")
            .args(["-p", "elisa", "-F", "status"])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .stdout
            .ok_or_else(|| Error::new(ErrorKind::Other, "could not capture the standard output."))
            .unwrap();
        let reader1 = BufReader::new(stdout1);

        reader1.lines().for_each(|status| {
            tx.send(status.unwrap()).unwrap();
        });
    });
    //Repurpose the following code for monitoring
    let stdout2 = Command::new("playerctl")
        .args(["-p", "elisa", "-F", "metadata", "--format", "{\"length\":\"{{duration(mpris:length)}}\",\"trackid\":\"{{mpris:trackid}}\",\"title\":\"{{xesam:title}}\"}"])
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture the standard output"))?;
    let reader2 = BufReader::new(stdout2);

    reader2
        .lines()
        //.filter_map(|line| {
        // println!("{:?}", line);
        //   Some(pattern.find(line.unwrap().as_str()).unwrap().clone())
        //})
        .for_each(|cap| {
            let metadata = serde_json::from_str::<Value>(cap.unwrap().clone().as_str()).unwrap();
            for received in &rx {
                println!("{received}");
                drpc.set_activity(|act| {
                    act.state(format!("Listening to {}", metadata["title"]))
                        .assets(|ass| ass.large_image("elisalogo"))
                        .details(received)
                })
                .expect("Could not set activity");
            }
        });
    //println!("{:?}", json_content);
    Ok(())
}
