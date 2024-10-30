#[derive(Debug)]
pub struct ReferenceTracker {
    immutable:bool,
    ref_count:usize
}

impl ReferenceTracker {

    pub fn new(immutable:bool) -> Self {
        Self {
            immutable,
            ref_count : 0
        }
    }

    pub fn modify_ref(&mut self, delta:isize) -> Result<ReferenceStatus, &str> {
        if self.immutable {
            Result::Ok(ReferenceStatus::Immutable)
        } else if delta < 0 {
            match self.ref_count.checked_sub(delta.abs() as usize) {
                Some(zero) if zero == 0 => {
                    self.ref_count = 0;
                    Result::Ok(ReferenceStatus::Zero)
                },
                Some(new_ref) => {
                    self.ref_count = new_ref;
                    Result::Ok(ReferenceStatus::Fine(self.ref_count))
                },
                None => {
                    Result::Err("Attempted to remove more references than instance has.")
                } 
            }     
        } else {
            self.ref_count += delta as usize;
            Result::Ok(ReferenceStatus::Fine(self.ref_count))
        }
    }

    pub fn get_status(&self) -> ReferenceStatus {
        if self.immutable { 
            ReferenceStatus::Immutable 
        } else if self.ref_count == 0 {
            ReferenceStatus::Zero
        } else {
            ReferenceStatus::Fine(self.ref_count)
        }
    }

}

#[derive(PartialEq)]
pub enum ReferenceStatus {
    Immutable,
    Fine(usize),
    Zero,
}