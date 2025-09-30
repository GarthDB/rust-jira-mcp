use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info, warn};

mod enhanced_mcp_tester;
mod fixture_collector;
mod fixture_test_runner;
mod mcp_tester;
mod test_runner;

use enhanced_mcp_tester::EnhancedMCPTester;
use fixture_collector::FixtureCollector;
use fixture_test_runner::FixtureTestRunner;
use mcp_tester::MCPTester;
use test_runner::TestRunner;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build tasks and utilities for rust-jira-mcp")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test MCP operations
    Test {
        /// Project key to test with
        #[arg(short = 'P', long, default_value = "DNA")]
        project: String,

        /// Test suite to run
        #[arg(short, long, default_value = "read-only")]
        suite: String,

        /// Use safe test project (TEST-MCP)
        #[arg(long)]
        safe: bool,

        /// Specific operation to test
        #[arg(short, long)]
        operation: Option<String>,

        /// JSON parameters for the operation
        #[arg(long)]
        params: Option<String>,
    },

    /// Enhanced test with real data patterns
    EnhancedTest {
        /// Test suite to run
        #[arg(short, long, default_value = "read-only")]
        suite: String,

        /// Use safe test project (TEST-MCP)
        #[arg(long)]
        safe: bool,
    },

    /// Test fixtures and generate test data
    FixtureTest {
        /// Test operation to run
        #[arg(short, long, default_value = "all")]
        operation: String,
    },

    /// Collect test fixtures from live API
    CollectFixtures {
        /// Project key to collect fixtures for
        #[arg(short, long, default_value = "DNA")]
        project: String,

        /// Output directory for fixtures
        #[arg(short, long, default_value = "tests/fixtures")]
        output: PathBuf,

        /// Anonymize sensitive data
        #[arg(long, default_value = "true")]
        anonymize: bool,
    },

    /// Generate synthetic test data
    GenerateFixtures {
        /// Output directory for fixtures
        #[arg(short, long, default_value = "tests/fixtures")]
        output: PathBuf,

        /// Number of issues to generate
        #[arg(short, long, default_value = "5")]
        count: usize,

        /// Project key for generated data
        #[arg(short, long, default_value = "TEST-MCP")]
        project: String,
    },

    /// Run comprehensive test suite
    TestSuite {
        /// Project key to test with
        #[arg(short, long, default_value = "TEST-MCP")]
        project: String,

        /// Skip cleanup (for debugging)
        #[arg(long)]
        no_cleanup: bool,

        /// Save detailed results
        #[arg(long)]
        save_results: bool,
    },

    /// Clean up test data
    Cleanup {
        /// Project key to clean up
        #[arg(short, long, default_value = "TEST-MCP")]
        project: String,

        /// Dry run (show what would be deleted)
        #[arg(long)]
        dry_run: bool,
    },

    /// Build the MCP server
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Run the MCP server
    Run {
        /// Configuration file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("xtask=debug,rust_jira_mcp=debug")
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            project,
            suite,
            safe,
            operation,
            params,
        } => {
            let tester = MCPTester::new();

            if let Some(op) = operation {
                // Test specific operation
                let params_json = if let Some(p) = params {
                    serde_json::from_str(&p)?
                } else {
                    serde_json::Map::new()
                };

                tester.test_operation(&op, params_json).await?;
            } else {
                // Run test suite
                let actual_project = if safe { "TEST-MCP" } else { &project };
                tester.run_test_suite(actual_project, &suite).await?;
            }
        }

        Commands::EnhancedTest { suite, safe } => {
            let tester = EnhancedMCPTester::new();

            match suite.as_str() {
                "read-only" => {
                    tester.run_enhanced_read_only_tests().await?;
                }
                "write" => {
                    let actual_project = if safe { "TEST-MCP" } else { "DNA" };
                    tester.run_enhanced_write_tests(actual_project).await?;
                }
                "bulk" => {
                    let actual_project = if safe { "TEST-MCP" } else { "DNA" };
                    tester.run_enhanced_bulk_tests(actual_project).await?;
                }
                "all" => {
                    tester.run_enhanced_read_only_tests().await?;
                    let actual_project = if safe { "TEST-MCP" } else { "DNA" };
                    tester.run_enhanced_write_tests(actual_project).await?;
                    tester.run_enhanced_bulk_tests(actual_project).await?;
                }
                _ => {
                    error!("Unknown enhanced test suite: {}", suite);
                    anyhow::bail!("Unknown enhanced test suite: {}", suite);
                }
            }
        }

        Commands::FixtureTest { operation } => {
            let runner = FixtureTestRunner::new();

            match operation.as_str() {
                "serialization" => {
                    runner.test_serialization_with_fixtures()?;
                }
                "anonymization" => {
                    runner.validate_anonymization()?;
                }
                "generate" => {
                    runner.generate_test_data_from_fixtures()?;
                }
                "all" => {
                    runner.run_fixture_tests().await?;
                }
                _ => {
                    error!("Unknown fixture test operation: {}", operation);
                    anyhow::bail!("Unknown fixture test operation: {}", operation);
                }
            }
        }

        Commands::CollectFixtures {
            project,
            output,
            anonymize,
        } => {
            let collector = FixtureCollector::new();
            collector
                .collect_fixtures(&project, &output, anonymize)
                .await?;
        }

        Commands::GenerateFixtures {
            output,
            count,
            project,
        } => {
            let collector = FixtureCollector::new();
            collector
                .generate_synthetic_fixtures(&project, &output, count)
                .await?;
        }

        Commands::TestSuite {
            project,
            no_cleanup,
            save_results,
        } => {
            let runner = TestRunner::new();
            runner
                .run_comprehensive_suite(&project, !no_cleanup, save_results)
                .await?;
        }

        Commands::Cleanup { project, dry_run } => {
            let runner = TestRunner::new();
            runner.cleanup_test_data(&project, dry_run).await?;
        }

        Commands::Build { release } => {
            build_mcp_server(release).await?;
        }

        Commands::Run { config } => {
            run_mcp_server(config).await?;
        }
    }

    Ok(())
}

async fn build_mcp_server(release: bool) -> Result<()> {
    info!("Building MCP server...");

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");

    if release {
        cmd.arg("--release");
        info!("Building in release mode");
    } else {
        info!("Building in debug mode");
    }

    let status = cmd.status()?;

    if status.success() {
        info!("✅ Build completed successfully");
    } else {
        error!("❌ Build failed");
        std::process::exit(1);
    }

    Ok(())
}

async fn run_mcp_server(config: Option<PathBuf>) -> Result<()> {
    info!("Starting MCP server...");

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run");
    cmd.arg("--bin");
    cmd.arg("rust-jira-mcp");

    if let Some(config_path) = config {
        cmd.arg("--");
        cmd.arg("--config");
        cmd.arg(&config_path);
        info!("Using config file: {:?}", config_path);
    }

    let status = cmd.status()?;

    if !status.success() {
        error!("❌ MCP server failed to start");
        std::process::exit(1);
    }

    Ok(())
}
