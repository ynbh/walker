use serde::Serialize;

#[derive(Serialize)]
pub struct FinalStatus {
    pub url: String,
    pub status: String,
}

pub fn parse(status: String) -> String {
    let lines = status.split("\n");
    let mut v: Vec<FinalStatus> = vec![];

    for values in lines {
        let c = values
            .split(": ")
            .into_iter()
            .filter(|x| *x != "")
            .collect::<Vec<&str>>();

        if c.len() > 0 {
            v.push(FinalStatus {
                url: c[0].to_string(),
                status: c[1].to_string(),
            })
        }
    }

    serde_json::to_string_pretty(&v).unwrap()
}
