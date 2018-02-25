use super::brush_preview_action::*;

use animation::*;

///
/// Represents an editing action for a tool
/// 
pub enum ToolAction<ToolData> {
    /// Changes the data that will be specified at the start of the next tool input stream
    Data(ToolData),

    /// Specifies an edit to perform
    Edit(AnimationEdit),

    /// Specifies a brush preview action to perform
    BrushPreview(BrushPreviewAction)
}