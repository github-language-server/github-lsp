use std::time::Instant;

use tokio::process::Command;

#[tokio::main]
async fn main() {
    let start = Instant::now();
    /*
    Showing 4 of 4 open issues in entur/helm-charts\n
    #142  Automatically update examples as part of release                about 4 months ago
    #126  `ingress.class` annotation is deprecated           enhancement  about 5 months ago
    #101  Use internalPort to specify k8s' grpc probe ports               about 4 months ago
    #42   Add integration test for cron job                  enhancement  about 1 year ago
    */
    let output = Command::new("gh")
        .arg("issue")
        .arg("list")
        .output()
        .await
        .unwrap()
        .stdout
        .to_owned();
    let output = String::from_utf8(output).unwrap();
    println!("{}", output);
    let iss: Vec<&str> = output.lines().collect();
    let mut compls: Vec<String> = vec![];
    for is in iss {
        let parts = is
            .split('\t')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let id = parts[0].clone();
        let label = parts
            .iter()
            .take(3)
            .map(|s| s.to_owned())
            .collect::<Vec<String>>()
            .join(" ")
            .to_owned();
        println!("{}", id);
        /*
        Add integration test for cron job #42
        Open • AlexanderBrevig opened about 1 year ago • 0 comments
        Labels: enhancement\n\nNo description provided\n\n
        View this issue on GitHub: https://github.com/entur/helm-charts/issues/42
        */
        let detail = Command::new("gh")
            .arg("issue")
            .arg("view")
            .arg(id)
            .output()
            .await
            .unwrap()
            .stdout
            .to_owned();
        let detail = String::from_utf8(detail).unwrap();
        compls.push(label);
        compls.push(detail);
    }
    println!("{:?}", compls);
    let duration = start.elapsed();

    println!("Time elapsed in expensive_function() is: {:?}", duration);
}
