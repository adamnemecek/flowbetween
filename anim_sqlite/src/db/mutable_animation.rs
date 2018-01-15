use super::*;

use std::collections::*;

///
/// Class used for the animation object for a database
/// 
pub struct AnimationEditor {
    /// The core, where the edits are sent
    core: Arc<Desync<AnimationDbCore>>,

    /// The layers created by this editor (used to track state)
    layers: HashMap<u64, SqliteVectorLayer>
}

impl AnimationEditor {
    ///
    /// Creates a new animation editor
    /// 
    pub fn new(core: &Arc<Desync<AnimationDbCore>>) -> AnimationEditor {
        AnimationEditor {
            core:   Arc::clone(core),
            layers: HashMap::new()
        }
    }

    ///
    /// Performs an edit on this item (if the core's error condition is clear)
    /// 
    fn edit<TEdit: Fn(&Connection, i64, &AnimationDbCore) -> Result<()>+Send+'static>(&mut self, edit: TEdit) {
        self.core.async(move |core| core.edit(edit))
    }
}

impl MutableAnimation for AnimationEditor {
    fn set_size(&mut self, size: (f64, f64)) {
        // Update the size for the current animation
        self.edit(move |sqlite, animation_id, _core| {
            sqlite.execute(
                "UPDATE Flo_Animation SET SizeX = ?, SizeY = ? WHERE AnimationId = ?",
                &[&size.0, &size.1, &animation_id]
            )?;

            Ok(())
        })
    }

    fn add_layer(&mut self, new_layer_id: u64) {
        // Create a layer with this assigned ID
        self.edit(move |sqlite, animation_id, _core| {
            // TODO: hard codes the layer type as 0 (vector layer), but we can't set layer types right now anyway
            // Create the layer
            let mut make_new_layer  = sqlite.prepare("INSERT INTO Flo_LayerType (LayerType) VALUES (0)")?;
            let layer_id            = make_new_layer.insert(&[])?;

            // Give it an assigned ID
            sqlite.execute(
                "INSERT INTO Flo_AnimationLayers (AnimationId, LayerId, AssignedLayerId) VALUES (?, ?, ?)",
                &[&animation_id, &layer_id, &(new_layer_id as i64)]
            )?;

            Ok(())
        })
    }

    fn remove_layer(&mut self, old_layer_id: u64) {
        // Remove the cached version of this layer
        self.layers.remove(&old_layer_id);

        // Create a layer with this assigned ID
        self.edit(move |sqlite, _animation_id, _core| {
            // Delete the layer with this assigned ID (triggers will clear out everything else)
            sqlite.execute(
                "DELETE FROM Flo_AnimationLayers WHERE AssignedLayerId = ?",
                &[&(old_layer_id as i64)]
            )?;

            Ok(())
        })
    }

    fn edit_layer<'a>(&'a mut self, layer_id: u64) -> Option<Editor<'a, Layer>> {
        // Create the layer if one is not already cached
        let layer = if !self.layers.contains_key(&layer_id) {
            let new_layer = SqliteVectorLayer::from_assigned_id(&self.core, layer_id);

            if let Some(new_layer) = new_layer {
                self.layers.insert(layer_id, new_layer);
                self.layers.get_mut(&layer_id)
            } else {
                None
            }
        } else {
            self.layers.get_mut(&layer_id)
        };

        // Edit it
        layer.map(|layer| Editor::new(layer as &mut Layer))
    }
}
