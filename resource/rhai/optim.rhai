fn sqlen(vec) {
    vec[0] * vec[0] + vec[1] * vec[1]
}

fn sub(a, b) {
    [a[0] - b[0], a[1] - b[1]]
}

fn sqdist(a, b) {
    sqlen(sub(a, b))
}

// Returns the distance between two points.
fn dist(a, b) {
    sqdist(a, b).sqrt()
}

let should_pass_all_frames = false;

fn is_valid(curr) {
    true
}

fn is_better(curr, best) {
    true
}

fn to_string(curr) {
    // you can reference to variables in is_better() here
    "".to_string()
}