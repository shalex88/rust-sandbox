#[derive(Clone, Debug)]
pub struct Order {
    pub customer_id: String,
    pub created_at: u8, // Simple data type for a demo purposes
}

pub fn recent_orders(
    orders: &Vec<Order>,
    customer_id: &String,
    limit: usize
) -> Vec<Order> {
    let mut matches: Vec<Order> = orders
        .iter()
        .filter(|order| order.customer_id == *customer_id)
        .cloned()
        .collect();

    matches.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    matches.truncate(limit);
    matches
}

// Copy only matching objects
pub fn better_recent_orders(
    orders: &[Order], // Accepts slices, including borrowed vectors and arrays
    customer_id: &str, // No need to own the string, also allows to pass the text as string literals
    limit: usize
) -> Vec<Order> {
    let mut matches: Vec<&Order> = orders // Store references and not objects
        .iter()
        .filter(|order| order.customer_id == *customer_id)
        .collect();

    matches.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    matches.truncate(limit);
    matches.into_iter().cloned().collect()
}

// No copy at all
// Add lifetime parameter to function definition.
// It connects the input and output references and tells rust that
// the returned references cannot outlive the orders they refer to.
pub fn even_better_recent_orders<'a>(
    orders: &'a [Order], // Add lifetime parameter
    customer_id: &str,
    limit: usize
) -> Vec<&'a Order> { // Return vector of references and add lifetime parameter
    let mut matches: Vec<&Order> = orders
        .iter()
        .filter(|order| order.customer_id == *customer_id)
        .collect();

    matches.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    matches.truncate(limit);
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_orders() -> Vec<Order> {
        vec![
            Order {
                customer_id: "alice".into(),
                created_at: 1,
            },
            Order {
                customer_id: "bob".into(),
                created_at: 5,
            },
            Order {
                customer_id: "alice".into(),
                created_at: 3,
            },
            Order {
                customer_id: "alice".into(),
                created_at: 2,
            },
        ]
    }

    fn assert_recent_orders(orders: Vec<Order>, customer_id: &str, limit: usize, expected: &[u8]) {
        let customer_id = customer_id.to_owned();
        let original = recent_orders(&orders, &customer_id, limit);
        let better = better_recent_orders(&orders, &customer_id, limit);
        let borrowed = even_better_recent_orders(&orders, &customer_id, limit);

        for result in [
            original.iter().collect::<Vec<_>>(),
            better.iter().collect(),
            borrowed,
        ] {
            assert!(result.iter().all(|order| order.customer_id == customer_id));
            let timestamps: Vec<_> = result.iter().map(|order| order.created_at).collect();
            assert_eq!(timestamps, expected);
        }
    }

    #[test]
    fn returns_latest_orders_for_requested_customer() {
        assert_recent_orders(sample_orders(), "alice", 2, &[3, 2]);
    }

    #[test]
    fn returns_empty_when_customer_has_no_orders() {
        assert_recent_orders(sample_orders(), "charlie", 2, &[]);
    }

    #[test]
    fn returns_empty_when_limit_is_zero() {
        assert_recent_orders(sample_orders(), "alice", 0, &[]);
    }

    #[test]
    fn returns_empty_for_empty_input() {
        assert_recent_orders(vec![], "alice", 2, &[]);
    }

    #[test]
    fn returns_all_matches_when_limit_exceeds_count() {
        assert_recent_orders(sample_orders(), "alice", 10, &[3, 2, 1]);
    }

    #[test]
    fn returns_references_to_original_orders() {
        let orders = sample_orders();
        let result = even_better_recent_orders(&orders, "alice", 2);

        assert_eq!(result.len(), 2);
        assert!(std::ptr::eq(result[0], &orders[2]));
        assert!(std::ptr::eq(result[1], &orders[3]));
    }
}
