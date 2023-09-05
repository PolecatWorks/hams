struct HealthProbe {
    /// vector that is shared across clones AND the objects it refers to can also be independantly shared
    detail: Arc<Mutex<HashSet<HealthCheck>>>,
    /// Previous assignment of alive to allow state change operations
    previous: Arc<AtomicBool>,
    /// enable alive reply or disable (for debug use)
    enabled: Arc<AtomicBool>,
}

impl HealthProbe {
    fn new() -> HealthProbe {
        info!("Constructing HealthProbe");

        HealthProbe {
            detail: Arc::new(Mutex::new(HashSet::new())),
            previous: Arc::new(AtomicBool::new(false)),
            enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    fn insert(&self, hc: HealthCheck) -> Result<(), HamsError> {
        println!("implement this");
        // self.detail
        //     .lock()
        //     .unwrap()
        //     .insert(hc);
        Ok(())
    }
    fn remove(&self, hc: &HealthCheck) -> Result<(), HamsError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, time::Duration};

    // #[test]
    // fn health_create() {

    //     let kick = HealthKick::new("black", Duration::from_secs(3));

    //     let hc0 = HealthCheck::for_hc(kick);

    //     // let ben = (*hc0).name;

    // }

    #[test]
    fn kick_create_and_destroy() {
        let probe = HealthProbe::new();
    }

    #[test]
    fn kick_sample_usage() {
        let hc = HealthKick::new("banana", Duration::from_secs(10));
        hc.kick();
    }

    #[test]
    fn construct_probe_and_populate() {
        let probe = HealthProbe::new();

        let hc0 = HealthKick::new("banana0", Duration::from_secs(10));
        let hc1 = HealthKick::new("banana1", Duration::from_secs(10));
        let hc2 = HealthKick::new("banana2", Duration::from_secs(10));

        let mut myvec = Vec::new();

        hc0.kick();
        myvec.push(&hc0);
        myvec.push(&hc1);
        myvec.push(&hc2);

        println!("myvec = {:?}", myvec);

        // probe.insert(&hc);

        hc0.kick();

        println!("myvec = {:?}", myvec);
        // probe.remove(&hc);
        let me = myvec.remove(0);
        drop(me);
        drop(myvec);
        drop(hc0);
        // println!("myvec = {:?}", myvec);

        // probe.insert(&hc);

        // let hc = HealthCheck::new();
    }
}
