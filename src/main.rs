mod order;
use order::Order;
use order::better_recent_orders;
use order::even_better_recent_orders;
use order::recent_orders;

fn main() {
    let orders = vec![
        Order {
            customer_id: String::from("alice"),
            created_at: 1,
        },
        Order {
            customer_id: String::from("bob"),
            created_at: 1,
        },
        Order {
            customer_id: String::from("alice"),
            created_at: 2,
        },
        Order {
            customer_id: String::from("alice"),
            created_at: 3,
        },
        Order {
            customer_id: String::from("bob"),
            created_at: 3,
        },
    ];

    {
        let customer_id = String::from("bob");
        let recent_orders = recent_orders(&orders, &customer_id, 3);
        println!(
            "Customer `{}` made {} recent orders",
            customer_id,
            recent_orders.len()
        );
        #[cfg(debug_assertions)]
        dbg!(&recent_orders);
    }
    {
        let customer_id = String::from("bob"); // Not mandatory, can use "bob" directly
        let recent_orders = better_recent_orders(&orders, &customer_id, 3);
        println!(
            "Customer `{}` made {} recent orders",
            customer_id,
            recent_orders.len()
        );
        #[cfg(debug_assertions)]
        dbg!(&recent_orders);
    }
    {
        let customer_id = String::from("bob"); // Not mandatory, can use "bob" directly
        let recent_orders = even_better_recent_orders(&orders, &customer_id, 3);
        println!(
            "Customer `{}` made {} recent orders",
            customer_id,
            recent_orders.len()
        );
        #[cfg(debug_assertions)]
        dbg!(&recent_orders);
    }
}
