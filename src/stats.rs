use std::fmt;

pub struct Stats {
    get_elapsed: String,
    loop_elapsed: String,
    link_count: usize,
}

impl Stats {
    pub fn new(get_elapsed: String, loop_elapsed: String, link_count: usize) -> Stats {
        Stats {
            get_elapsed,
            loop_elapsed,
            link_count,
        }
    }

    pub fn to_markdown(&self) -> String {
        format!("## Stats \nTime to get {0} links: **{1}** seconds\n\nTime to verify {0} links: **{2}** seconds", self.link_count, self.get_elapsed, self.loop_elapsed)
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = format!("Stats:");
        let get_elapsed_message = format!(
            "Time to get {} links: {} seconds",
            self.link_count, self.get_elapsed
        );
        let verify_elapsed_message = format!(
            "Time to verify {} links: {} seconds",
            self.link_count, self.loop_elapsed
        );

        write!(
            f,
            "{title}\n{get_elapsed_message}\n{verify_elapsed_message}"
        )
    }
}
