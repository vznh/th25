use octocrab::Octocrab;
use octocrab::models::pulls::PullRequest;
use octocrab::params::State;

/// Initialize Octocrab with a GitHub installation token.
pub fn init_octocrab(installation_token: String) -> Octocrab {
  Octocrab::builder()
    .personal_token(installation_token)
    .build()
    .expect("Failed to initialize Octocrab")
}

/// List open pull requests in a repository.
pub async fn test_list_pull_requests(octo: &Octocrab, owner: &str, repo: &str) {
  match octo.pulls(owner, repo).list().state(State::Open).per_page(5).send().await {
    Ok(prs) => {
      let pr_list: Vec<PullRequest> = prs.items;
      println!("Open pull requests in {}/{}: {:#?}", owner, repo, pr_list);
    }
    Err(err) => {
      println!("Failed to list pull requests: {:?}", err);
    }
  }
}

pub async fn reply_to_latest_pr(octo: &Octocrab, owner: &str, repo: &str) {
  match octo.pulls(owner, repo).list().state(State::Open).per_page(1).send().await {
    Ok(prs) => {
      if let Some(pr) = prs.items.first() {
        let pr_number = pr.number;
        println!("Found latest PR: #{}", pr_number);

        // Step 2: Post a comment with "hej"
        match octo.issues(owner, repo).create_comment(pr_number, "hej").await {
          Ok(comment) => println!("Comment posted: {}", comment.html_url.to_string()),
          Err(err) => println!("Failed to comment on PR #{}: {:?}", pr_number, err),
        }
      } else {
        println!("No open PRs found.");
      }
    }
    Err(err) => {
      println!("Failed to fetch PRs: {:?}", err);
    }
  }
}
