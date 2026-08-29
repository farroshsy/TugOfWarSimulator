use crate::strategies::Allocation;

/// Generate all possible allocations of P people among R ropes
/// 
/// For P people and R ropes, the number of allocations is:
/// C(P + R - 1, R - 1)
/// 
/// For example, with P=12, R=3: C(14,2) = 91 allocations
pub fn generate_allocations(people: usize, ropes: usize) -> Vec<Allocation> {
    if ropes == 0 {
        return vec![];
    }
    
    let mut result = Vec::new();
    let mut current = Allocation::new(ropes);
    
    generate_recursive(0, people, &mut current, &mut result);
    
    result
}

fn generate_recursive(
    index: usize,
    remaining: usize,
    current: &mut Allocation,
    result: &mut Vec<Allocation>,
) {
    if index == current.counts.len() - 1 {
        // Last rope gets all remaining people
        current.counts[index] = remaining;
        result.push(current.clone());
        return;
    }
    
    // Try all possible counts for this rope (0 to remaining)
    for count in 0..=remaining {
        current.counts[index] = count;
        generate_recursive(index + 1, remaining - count, current, result);
    }
}

/// Calculate the number of possible allocations using the combinatorial formula
/// C(P + R - 1, R - 1)
pub fn count_allocations(people: usize, ropes: usize) -> usize {
    if ropes == 0 {
        return 0;
    }
    
    binomial_coefficient(people + ropes - 1, ropes - 1)
}

/// Calculate binomial coefficient C(n, k)
fn binomial_coefficient(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    if k == 0 || k == n {
        return 1;
    }
    
    // Use the smaller k for efficiency
    let k = k.min(n - k);
    
    let mut result = 1;
    for i in 0..k {
        result = result * (n - i) / (i + 1);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_allocations_small() {
        let allocations = generate_allocations(3, 2);
        // C(3+2-1, 2-1) = C(4,1) = 4 allocations
        assert_eq!(allocations.len(), 4);
        
        // Check some specific allocations
        assert!(allocations.iter().any(|a| a.counts == vec![3, 0]));
        assert!(allocations.iter().any(|a| a.counts == vec![0, 3]));
        assert!(allocations.iter().any(|a| a.counts == vec![1, 2]));
        assert!(allocations.iter().any(|a| a.counts == vec![2, 1]));
    }
    
    #[test]
    fn test_count_allocations() {
        assert_eq!(count_allocations(3, 2), 4);
        assert_eq!(count_allocations(12, 3), 91); // C(14,2) = 91
        assert_eq!(count_allocations(4, 3), 15); // C(6,2) = 15
    }
    
    #[test]
    fn test_allocation_sum() {
        let allocations = generate_allocations(5, 3);
        for allocation in &allocations {
            let sum: usize = allocation.counts.iter().sum();
            assert_eq!(sum, 5);
        }
    }
    
    #[test]
    fn test_allocation_correctness() {
        let allocations = generate_allocations(2, 2);
        // Should have: [2,0], [1,1], [0,2]
        assert_eq!(allocations.len(), 3);
        
        let sums: Vec<usize> = allocations.iter().map(|a| a.counts.iter().sum()).collect();
        assert!(sums.iter().all(|&s| s == 2));
    }
}