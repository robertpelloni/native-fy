use wgpu::GlobalReport;

fn main() {
    let type_name = std::any::type_name::<GlobalReport>();
    println!("{}", type_name);
}
