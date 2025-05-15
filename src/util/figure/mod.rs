use std::collections::HashMap;
use std::collections::HashSet;

pub struct FigureUtil;

impl FigureUtil {
    // Get figure parts from a figure string
    pub fn get_figure_bits(looks: &str) -> HashMap<String, String> {
        let mut bits = HashMap::new();
        let sets = looks.split('.');
        
        for set in sets {
            let set_bits: Vec<&str> = set.split('-').collect();
            if set_bits.len() > 0 {
                let key = set_bits[0].to_string();
                let value = if set_bits.len() > 1 { set_bits[1].to_string() } else { String::new() };
                bits.insert(key, value);
            }
        }
        
        bits
    }
    
    // Merge two figure strings
    pub fn merge_figures(figure1: &str, figure2: &str) -> String {
        Self::merge_figures_with_limits(figure1, figure2, None, None)
    }
    
    // Merge two figure strings with limits on the first figure
    pub fn merge_figures_with_limit_1(figure1: &str, figure2: &str, limit_figure1: Option<&[String]>) -> String {
        Self::merge_figures_with_limits(figure1, figure2, limit_figure1, None)
    }
    
    // Check if figure has blacklisted clothing items
    pub fn has_blacklisted_clothing(figure: &str, blacklist: &HashSet<i32>) -> bool {
        for set in figure.split('.') {
            let pieces: Vec<&str> = set.split('-').collect();
            
            if pieces.len() >= 2 {
                if let Ok(piece_id) = pieces[1].parse::<i32>() {
                    if blacklist.contains(&piece_id) {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    // Merge two figure strings with limits on both figures
    pub fn merge_figures_with_limits(
        figure1: &str, 
        figure2: &str, 
        limit_figure1: Option<&[String]>, 
        limit_figure2: Option<&[String]>
    ) -> String {
        let figure_bits1 = Self::get_figure_bits(figure1);
        let figure_bits2 = Self::get_figure_bits(figure2);
        
        let mut final_look = String::new();
        
        // Add parts from figure1
        for (key, value) in &figure_bits1 {
            if limit_figure1.is_none() || limit_figure1.unwrap().contains(&key) {
                final_look.push_str(&format!("{}-{}." , key, value));
            }
        }
        
        // Add parts from figure2
        for (key, value) in &figure_bits2 {
            if limit_figure2.is_none() || limit_figure2.unwrap().contains(&key) {
                final_look.push_str(&format!("{}-{}." , key, value));
            }
        }
        
        // Remove trailing dot if present
        if final_look.ends_with('.') {
            final_look.pop();
        }
        
        final_look
    }
}