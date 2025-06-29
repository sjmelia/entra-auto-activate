use std::error::Error;
use std::process::Command;
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args
{
    #[arg(short, long)]
    role_name: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Args::parse();

    for role_name in &args.role_name {
        println!("{:?}", role_name);
    }

    let mut process = match Command::new("msedgedriver")
        .args(&["--port=54950"])
        .spawn() {
        Ok(process) => process,
        Err(err) => panic!("Running process error: {}", err),
    };

    let caps = DesiredCapabilities::edge();
    let driver = WebDriver::new("http://localhost:54950", caps).await?;
    driver.goto("https://entra.microsoft.com/#view/Microsoft_Azure_PIMCommon/GroupRoleBlade/resourceId//subjectId//isInternalCall~/true?Microsoft_AAD_IAM_legacyAADRedirect=true/").await?;
    let roles_table_tbody = driver.query(By::ClassName("azc-grid-groupdata")).first().await?;
    let role_rows = roles_table_tbody.query(By::Tag("tr")).all_from_selector().await?;
    
    for row in role_rows {
        let columns = row.query(By::Tag("td")).all_from_selector().await?;
        let role_description_column = &columns[1];
        let cell_content_div = role_description_column.find(By::Tag("div")).await?;
        let viva_control_div = cell_content_div.find(By::Tag("div")).await?;
        let content_div = viva_control_div.find(By::Tag("div")).await?;
        let content = content_div.text().await?;

        println!("{:?}", content);
        if args.role_name.contains(&content) {
            let activate_column = &columns[5];
            let activate_link = activate_column.query(By::Tag("a")).first().await?;
            activate_link.click().await?;

            let slider = driver.query(By::ClassName("fxc-slider")).first().await?;
            let tab_pane = slider.parent().await?;

            let tab_pane_divs = tab_pane.query(By::Tag("div")).all_from_selector().await?;

            for div in tab_pane_divs {
                let data_formelement = div.attr("data-formelement").await?;
                match data_formelement {
                    Some(str) => match str.as_str() {
                        "pcControl: ticketSystemTextBox" => {
                            let input = div.query(By::Tag("input")).first().await?;
                            input.wait_until().enabled().await?;
                            input.send_keys("1").await?;
                        },
                        "pcControl: ticketNumberTextBox" => {
                            let input = div.query(By::Tag("input")).first().await?;
                            input.wait_until().enabled().await?;
                            input.send_keys("1").await?;
                        },
                         "pcControl: comments" => {
                            let input = div.query(By::Tag("textarea")).first().await?;
                            input.wait_until().enabled().await?;
                            input.send_keys("1").await?;
                        },
                        _ => {}
                    }
                    _ => continue
                }
            }

            let container = tab_pane.parent().await?.parent().await?.parent().await?.parent().await?.parent().await?;
            println!("{:?}", container.attr("class").await?);
            let buttons = container.query(By::ClassName("fxs-button")).all_from_selector().await?;
            for button in buttons {
                println!("{:?}", button.inner_html().await?);

                let title_option = button.attr("title").await?;
                
                if let Some(title) = title_option {
                    if title == "Activate" {
                        button.wait_until().enabled().await?;
                        button.click().await?;
                        sleep(Duration::from_millis(10000)).await;
                        break;
                    }
                }
            }
        }
    }

    driver.quit().await?;
    let _ = process.kill();
    Ok(())
}