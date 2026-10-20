fn main() {
    println!("cargo:rerun-if-changed=assets/index.html");
    println!("cargo:rerun-if-changed=assets/customer.html");
    println!("cargo:rerun-if-changed=assets/css/style.css");
    println!("cargo:rerun-if-changed=assets/css/customer.css");
    println!("cargo:rerun-if-changed=assets/js/api.js");
    println!("cargo:rerun-if-changed=assets/js/cart.js");
    println!("cargo:rerun-if-changed=assets/js/terminal.js");
    println!("cargo:rerun-if-changed=assets/js/views.js");
    println!("cargo:rerun-if-changed=assets/js/app.js");
    println!("cargo:rerun-if-changed=assets/js/customer.js");
    println!("cargo:rerun-if-changed=assets/images/caddecafe_logo.webp");
    println!("cargo:rerun-if-changed=assets/images/logo.webp");
    println!("cargo:rerun-if-changed=assets/images/pofuduk-logo.svg");
}
