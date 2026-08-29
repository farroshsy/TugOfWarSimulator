use serde::{Deserialize, Serialize};

pub type RopeId = usize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rope {
    pub id: RopeId,
    
    // Position: negative = Team A territory, positive = Team B territory, zero = center
    pub position: i32,
    pub min_position: i32,
    pub max_position: i32,
    
    // Physical properties
    pub weight: f64,
    pub value: i32,
    
    // Current player counts (for visualization)
    pub team_a_count: usize,
    pub team_b_count: usize,
}

impl Rope {
    pub fn new(
        id: RopeId,
        position: i32,
        min_position: i32,
        max_position: i32,
        weight: f64,
        value: i32,
    ) -> Self {
        Self {
            id,
            position: position.clamp(min_position, max_position),
            min_position,
            max_position,
            weight,
            value,
            team_a_count: 0,
            team_b_count: 0,
        }
    }
    
    pub fn is_in_team_a_territory(&self) -> bool {
        self.position < 0
    }
    
    pub fn is_in_team_b_territory(&self) -> bool {
        self.position > 0
    }
    
    pub fn is_at_center(&self) -> bool {
        self.position == 0
    }
    
    pub fn distance_from_center(&self) -> i32 {
        self.position.abs()
    }
}