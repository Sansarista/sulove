use std::backtrace::Backtrace;

pub struct DebugUtils;

impl DebugUtils {
    // Get the caller's caller stack trace information
    pub fn get_caller_caller_stacktrace() -> Option<String> {
        let backtrace = Backtrace::capture();
        let backtrace_str = format!("{:?}", backtrace);
        
        // Parse the backtrace to find the caller's caller
        // This is a simplified version as Rust's backtrace API differs from Java's
        let frames: Vec<&str> = backtrace_str.lines().collect();
        
        // Skip the first few frames which are this function and its callers
        // The exact number to skip may need adjustment based on testing
        if frames.len() > 3 {
            return Some(frames[3].to_string());
        }
        
        None
    }
}