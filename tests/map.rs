use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::json;

#[test]
fn regular() {
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Test {
        #[serde(with = "serde_extra::ts_secs::option")]
        seconds: Option<Duration>,
        #[serde(with = "serde_extra::ts_millis")]
        millis: Duration,
    }

    let time = json! {{
        "seconds": 120,
        "millis": 10,
    }};

    assert_eq!(
        serde_json::to_string(&time)
            .and_then(|x| serde_json::from_str::<Test>(&x))
            .unwrap(),
        Test {
            seconds: Some(Duration::from_secs(120)),
            millis: Duration::from_millis(10)
        }
    );

    // let time = json! {{
    //     "seconds": None,
    //     "millis": 10,
    // }};
}
