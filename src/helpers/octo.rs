use octocrab::Octocrab;
use octocrab::models::pulls::PullRequest;
use octocrab::params::{State, repos::Reference};
use std::error::Error;

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

//// Find the latest PR and create a unique "mechanic-[issue]" branch from its latest commit.
pub async fn create_mechanic_branch(octo: &Octocrab, owner: &str, repo: &str) {
  // Step 1: Find the latest PR
  match octo.pulls(owner, repo).list().state(State::Open).per_page(1).send().await {
    Ok(prs) => {
      if let Some(pr) = prs.items.first() {
        let pr_number = pr.number;
        println!("Found latest PR: #{}", pr_number);

        // Step 2: Get the latest commit SHA of the PR
        let latest_commit = pr.head.sha.clone();

        println!("Latest PR commit SHA: {}", latest_commit);

        // Step 3: Generate base branch name
        let base_branch = format!("mechanic-{}", pr_number);
        let mut new_branch = base_branch.clone();
        let mut counter = 1;

        // Step 4: Check if the branch exists and increment if necessary
        while octo.repos(owner, repo).get_ref(&Reference::Branch(new_branch.clone())).await.is_ok()
        {
          new_branch = format!("{}-{}", base_branch, counter);
          counter += 1;
        }

        // Step 5: Create a new branch from the latest commit
        match octo
          .repos(owner, repo)
          .create_ref(&Reference::Branch(new_branch.clone()), &latest_commit)
          .await
        {
          Ok(_) => println!("Created new branch: {}", new_branch),
          Err(err) => println!("Failed to create new branch: {:?}", err),
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

pub async fn reply_to_latest_pr(octo: &Octocrab, owner: &str, repo: &str) {
  match octo.pulls(owner, repo).list().state(octocrab::params::State::Open).per_page(1).send().await
  {
    Ok(prs) => {
      if let Some(pr) = prs.items.first() {
        let pr_number = pr.number;
        println!("Found latest PR: #{}", pr_number);
        match octo
          .issues(owner, repo)
          .create_comment(pr_number, "Mechanic doesn't have any suggestions to do. Great work!")
          .await
        {
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


pub async fn post_markdown_as_comment(
  octo: &Octocrab,
  owner: &str,
  repo: &str,
  pr_number: u64,
  markdown: &str,
) -> Result<(), Box<dyn Error>> {
  let comment = octo
      .issues(owner, repo)
      .create_comment(pr_number, markdown)
      .await?;
  println!("Comment posted: {}", comment.html_url);
  Ok(())
}