use super::*;


pub fn set_grid_cell(to:ExternalPointer, world_point:Vec2, world:&mut World, graph:&mut SparseDirectedGraph) {
    let target = world.query_mut::<(&mut Location, &mut Editing)>();
    for (_, (location, _)) in target {
        let height = to.height;
        if height <= location.pointer.height {
            let cell = Gate::point_to_cells(location, height, world_point)[0];
            if let Some(cell) = cell {
                let path = ZorderPath::from_cell(cell, location.pointer.height - height);
                if let Ok(pointer) = graph.set_node(location.pointer, &path.steps(), to.pointer) {
                    location.pointer = pointer;
                } else {dbg!("Write failure. That's really bad.");}
            }
        }
    }
}

/*
pub fn expand_object_domain(&mut self, object_index:usize, direction:usize) {
    let object = &mut self.objects[object_index];
    //Prevent zorder overflow for now
    if object.root.height == 15 { dbg!("We don't overflow around here"); return }
    object.position += object.cell_length(0) * zorder_to_direction(direction as u32)/2.;
    let new_root = self.graph.set_node(NodePointer::new(Index(0)), &[direction as u32], object.root.pointer).unwrap();
    self.graph.swap_root(object.root.pointer, new_root);
    object.root.pointer = new_root;
    object.root.height += 1;
}

pub fn shrink_object_domain(&mut self, object_index:usize, preserve_direction:usize) {
    let object = &mut self.objects[object_index];
    if object.root.height == 0 { return }
    object.position += object.cell_length(0) * -zorder_to_direction(preserve_direction as u32)/4.;
    let new_root = self.graph.set_node(object.root.pointer, &[], self.graph.child(object.root.pointer, preserve_direction).unwrap()).unwrap();
    self.graph.swap_root(object.root.pointer, new_root);
    object.root.pointer = new_root;
    object.root.height -= 1;
}
*/
