use crate::error::ADBError;
use crate::ADB;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Automation features for batch operations and workflows
impl ADB {
    /// Execute a batch of ADB commands from a file
    pub fn execute_batch_file(&self, device: &str, batch_file_path: &str) -> Result<BatchResult, ADBError> {
        let content = fs::read_to_string(batch_file_path)?;
        let commands: Vec<&str> = content.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        self.execute_batch_commands(device, &commands)
    }

    /// Execute a batch of commands
    pub fn execute_batch_commands(&self, device: &str, commands: &[&str]) -> Result<BatchResult, ADBError> {
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        for command in commands {
            let start_time = std::time::Instant::now();
            match self.run_adb(&format!("-s {} {}", device, command)) {
                Ok(output) => {
                    success_count += 1;
                    results.push(CommandResult {
                        command: command.to_string(),
                        success: true,
                        output: Some(output),
                        error: None,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                    });
                }
                Err(e) => {
                    failure_count += 1;
                    results.push(CommandResult {
                        command: command.to_string(),
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                    });
                }
            }
        }

        Ok(BatchResult {
            total_commands: commands.len(),
            successful: success_count,
            failed: failure_count,
            results,
        })
    }

    /// Execute a workflow defined in a JSON file
    pub fn execute_workflow(&self, workflow_path: &str) -> Result<WorkflowResult, ADBError> {
        let content = fs::read_to_string(workflow_path)?;
        let workflow: Workflow = serde_json::from_str(&content)?;

        self.execute_workflow_definition(workflow)
    }

    /// Execute a workflow definition
    pub fn execute_workflow_definition(&self, workflow: Workflow) -> Result<WorkflowResult, ADBError> {
        let mut results = HashMap::new();
        let mut success = true;

        for step in workflow.steps {
            let step_result = match step.step_type {
                StepType::Command => self.execute_command_step(&step)?,
                StepType::Batch => self.execute_batch_step(&step)?,
                StepType::Conditional => self.execute_conditional_step(&step, &results)?,
                StepType::Parallel => self.execute_parallel_step(&step)?,
            };

            results.insert(step.name.clone(), step_result.clone());

            if !step_result.success && !step.continue_on_failure {
                success = false;
                break;
            }
        }

        Ok(WorkflowResult {
            workflow_name: workflow.name,
            success,
            steps_executed: results.len(),
            results,
        })
    }

    /// Create and execute a simple automation script
    pub fn run_automation_script(&self, script: AutomationScript) -> Result<AutomationResult, ADBError> {
        let mut results = Vec::new();

        for task in script.tasks {
            match task.task_type {
                TaskType::DeviceSetup => {
                    let result = self.setup_device_for_testing(&task.device)?;
                    results.push(AutomationTaskResult {
                        task_name: task.name,
                        success: result,
                        message: if result { "Device setup completed".to_string() } else { "Device setup failed".to_string() },
                    });
                }
                TaskType::AppInstallation => {
                    if let Some(app_path) = &task.app_path {
                        let result = self.install_app(&task.device, app_path).is_ok();
                        results.push(AutomationTaskResult {
                            task_name: task.name,
                            success: result,
                            message: if result { "App installed successfully".to_string() } else { "App installation failed".to_string() },
                        });
                    }
                }
                TaskType::DataCollection => {
                    let result = self.collect_device_data(&task.device)?;
                    results.push(AutomationTaskResult {
                        task_name: task.name,
                        success: true,
                        message: format!("Data collected: {} packages, {} processes", result.package_count, result.process_count),
                    });
                }
                TaskType::PerformanceTest => {
                    let result = self.run_performance_test(&task.device, task.duration_secs.unwrap_or(30))?;
                    results.push(AutomationTaskResult {
                        task_name: task.name,
                        success: true,
                        message: format!("Performance test completed - Avg CPU: {:.1}%, Memory: {}KB",
                                       result.avg_cpu, result.avg_memory),
                    });
                }
            }
        }

        let success = results.iter().all(|r| r.success);
        Ok(AutomationResult {
            script_name: script.name,
            success,
            tasks_completed: results.len(),
            results,
        })
    }

    // Helper methods for workflow steps
    fn execute_command_step(&self, step: &WorkflowStep) -> Result<StepResult, ADBError> {
        let start_time = std::time::Instant::now();
        let command = step.parameters.get("command")
            .ok_or_else(|| ADBError::InvalidArgument("Missing command parameter".to_string()))?;

        match self.run_adb(command) {
            Ok(output) => Ok(StepResult {
                step_name: step.name.clone(),
                success: true,
                output: Some(output),
                error: None,
                duration_ms: start_time.elapsed().as_millis() as u64,
            }),
            Err(e) => Ok(StepResult {
                step_name: step.name.clone(),
                success: false,
                output: None,
                error: Some(e.to_string()),
                duration_ms: start_time.elapsed().as_millis() as u64,
            }),
        }
    }

    fn execute_batch_step(&self, step: &WorkflowStep) -> Result<StepResult, ADBError> {
        let device = step.parameters.get("device")
            .ok_or_else(|| ADBError::InvalidArgument("Missing device parameter".to_string()))?;
        let commands_str = step.parameters.get("commands")
            .ok_or_else(|| ADBError::InvalidArgument("Missing commands parameter".to_string()))?;
        let commands: Vec<&str> = commands_str.split(';').map(|s| s.trim()).collect();

        let batch_result = self.execute_batch_commands(device, &commands)?;
        let success = batch_result.failed == 0;

        Ok(StepResult {
            step_name: step.name.clone(),
            success,
            output: Some(format!("Batch completed: {}/{} successful", batch_result.successful, batch_result.total_commands)),
            error: if success { None } else { Some(format!("{} commands failed", batch_result.failed)) },
            duration_ms: batch_result.results.iter().map(|r| r.duration_ms).sum(),
        })
    }

    fn execute_conditional_step(&self, step: &WorkflowStep, previous_results: &HashMap<String, StepResult>) -> Result<StepResult, ADBError> {
        let condition_step = step.parameters.get("condition_step")
            .ok_or_else(|| ADBError::InvalidArgument("Missing condition_step parameter".to_string()))?;
        let true_command = step.parameters.get("true_command");
        let false_command = step.parameters.get("false_command");

        let condition_success = previous_results.get(condition_step)
            .map(|r| r.success)
            .unwrap_or(false);

        let command = if condition_success {
            true_command
        } else {
            false_command
        };

        if let Some(cmd) = command {
            self.execute_command_step(&WorkflowStep {
                name: step.name.clone(),
                step_type: StepType::Command,
                parameters: HashMap::from([("command".to_string(), cmd.clone())]),
                continue_on_failure: step.continue_on_failure,
            })
        } else {
            Ok(StepResult {
                step_name: step.name.clone(),
                success: true,
                output: Some("Conditional step skipped".to_string()),
                error: None,
                duration_ms: 0,
            })
        }
    }

    fn execute_parallel_step(&self, step: &WorkflowStep) -> Result<StepResult, ADBError> {
        // Simplified parallel execution - in practice, you'd use tokio::spawn
        let commands_str = step.parameters.get("commands")
            .ok_or_else(|| ADBError::InvalidArgument("Missing commands parameter".to_string()))?;
        let commands: Vec<&str> = commands_str.split(';').map(|s| s.trim()).collect();

        let mut success_count = 0;
        let start_time = std::time::Instant::now();

        for command in &commands {
            if self.run_adb(command).is_ok() {
                success_count += 1;
            }
        }

        let success = success_count == commands.len();
        Ok(StepResult {
            step_name: step.name.clone(),
            success,
            output: Some(format!("Parallel execution: {}/{} successful", success_count, commands.len())),
            error: if success { None } else { Some(format!("{} commands failed", commands.len() - success_count)) },
            duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    // Helper methods for automation tasks
    fn setup_device_for_testing(&self, device: &str) -> Result<bool, ADBError> {
        // Enable developer options, disable animations, etc.
        let _ = self.set_animation_scale(device, 0.0);
        let _ = self.run_adb(&format!("-s {} shell settings put global development_settings_enabled 1", device));
        Ok(true)
    }

    fn collect_device_data(&self, device: &str) -> Result<DeviceData, ADBError> {
        let packages = self.get_package_list(device)?;
        let processes = self.get_running_processes(device)?;

        Ok(DeviceData {
            package_count: packages.len(),
            process_count: processes.len(),
        })
    }

    fn run_performance_test(&self, device: &str, duration_secs: u32) -> Result<PerformanceMetrics, ADBError> {
        let mut cpu_readings = Vec::new();
        let mut memory_readings = Vec::new();

        for _ in 0..duration_secs {
            if let Ok(profile) = self.get_performance_profile(device) {
                cpu_readings.push(profile.cpu_usage);
                memory_readings.push(profile.memory_usage);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        let avg_cpu = cpu_readings.iter().sum::<f32>() / cpu_readings.len() as f32;
        let avg_memory = (memory_readings.iter().sum::<u64>() / memory_readings.len() as u64) as u64;

        Ok(PerformanceMetrics {
            avg_cpu,
            avg_memory,
            duration_secs,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total_commands: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<CommandResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub step_type: StepType,
    pub parameters: HashMap<String, String>,
    pub continue_on_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    Command,
    Batch,
    Conditional,
    Parallel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_name: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_name: String,
    pub success: bool,
    pub steps_executed: usize,
    pub results: HashMap<String, StepResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationScript {
    pub name: String,
    pub tasks: Vec<AutomationTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationTask {
    pub name: String,
    pub task_type: TaskType,
    pub device: String,
    pub app_path: Option<String>,
    pub duration_secs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    DeviceSetup,
    AppInstallation,
    DataCollection,
    PerformanceTest,
}

#[derive(Debug)]
pub struct AutomationResult {
    pub script_name: String,
    pub success: bool,
    pub tasks_completed: usize,
    pub results: Vec<AutomationTaskResult>,
}

#[derive(Debug)]
pub struct AutomationTaskResult {
    pub task_name: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug)]
struct DeviceData {
    package_count: usize,
    process_count: usize,
}

#[derive(Debug)]
struct PerformanceMetrics {
    avg_cpu: f32,
    avg_memory: u64,
    duration_secs: u32,
}
