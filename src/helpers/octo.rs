// shove.rs
// meant for push events
use octocrab::Octocrab;
use octocrab::models::pulls::PullRequest;
use octocrab::params::State;

pub fn init_octocrab(installation_token: String) -> Octocrab {
  Octocrab::builder()
  .personal_token(installation_token)
  .build()
  .expect("Failed to initialize Octocrab")
}

pub async fn test_list_pull_requests(octo: &octocrab::Octocrab, owner: &str, repo: &str) {
  match octo
      .pulls(owner, repo)
      .list()
      .state(State::Open)
      .per_page(5)
      .send()
      .await
  {
      Ok(prs) => {
          let pr_list: Vec<PullRequest> = prs.items;
          println!("Open pull requests in {}/{}: {:#?}", owner, repo, pr_list);
      }
      Err(err) => {
          println!("Failed to list pull requests: {:?}", err);
      }
  }
}