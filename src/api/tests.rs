use super::*;

#[test]
fn test_calc_num_requets() {
    assert_eq!(
        calc_num_requests(
            "2020-01-01T00:00:00-04:00",
            "2020-01-01T00:00:01-04:00",
            300
        ),
        1
    );
    assert_eq!(
        calc_num_requests(
            "2020-01-01T00:00:00-04:00",
            "2020-01-01T00:05:00-04:00",
            300
        ),
        1
    );
    assert_eq!(
        calc_num_requests(
            "2020-01-01T00:00:00-04:00",
            "2020-01-02T00:23:55-04:00",
            300
        ),
        1
    );
    assert_eq!(
        calc_num_requests(
            "2020-01-01T00:00:00-04:00",
            "2020-01-02T01:00:00-04:00",
            300
        ),
        2
    );
}
