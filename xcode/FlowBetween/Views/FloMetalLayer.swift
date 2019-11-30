//
//  FloMetalLayer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 29/11/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa
import Metal

///
/// A layer used for drawing with OS X's metal protocol
///
class FloMetalLayer : CAMetalLayer {
    /// Function called to trigger a redraw
    fileprivate var _triggerRedraw: ((NSSize, NSRect) -> ())?;

    ///
    /// Sets the function to call when the layer needs to be redrawn
    ///
    func onRedraw(_ redraw: @escaping ((NSSize, NSRect) -> ())) {
        _triggerRedraw = redraw;
    }

    ///
    /// Updates the area of the canvas that this layer should display
    ///
    func setVisibleArea(bounds: ContainerBounds, resolution: CGFloat) {
        autoreleasepool {
            // Update the scale
            contentsScale       = resolution;
            
            // Trigger a redraw so the display is up to date
            _triggerRedraw?(bounds.totalSize, bounds.visibleRect);

            // Cause a redisplay
            CATransaction.begin();
            CATransaction.setAnimationDuration(0.0);
            CATransaction.disableActions();
            displayIfNeeded();
            CATransaction.commit();
        }
    }
}
