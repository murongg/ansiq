use ansiq_core::{computed, effect, signal};

fn main() {
    let count = signal(1u8);
    let doubled = computed({
        let count = count.clone();
        move || count.get() * 2
    });
    let effect = effect({
        let count = count.clone();
        move || {
            let _ = count.get();
        }
    });

    tokio::spawn(async move {
        count.set(2);
        let _ = doubled.get();
        effect.stop();
    });
}
