use super::*;


pub fn set_grid_cell(to:ExternalPointer, world_point:Vec2, world:&mut World, graph:&mut SparseDirectedGraph) {
    let target = world.query_mut::<(&mut Location, &mut Editing)>();
    for (_, (location, _)) in target {
        let height = to.height;
        if height <= location.pointer.height {
            let cell = Gate::point_to_cells(location.position, height, location.pointer.height, world_point)[0];
            if let Some(cell) = cell {
                let path = ZorderPath::from_cell(cell, location.pointer.height - height);
                if let Ok(pointer) = graph.set_node(location.pointer, &path.steps(), to.pointer) {
                    location.pointer = pointer;
                }
            }
        }
    }
}