//
//  FloView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 03/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation
import Cocoa

///
/// Class used to manage a view in FlowBetween
///
public class FloView : NSObject {
    /// The view that this represents
    fileprivate var _view: FloContainerView;
    
    /// The view that this is a subview of
    fileprivate weak var _superview: FloView?;
    
    /// The control contained by this view
    fileprivate var _control: NSControl!;
    
    /// The layout bounds of this view
    fileprivate var _bounds: Bounds;
    
    /// The subviews of this view
    fileprivate var _subviews: [FloView];
    
    /// Set to true if we've queued up a relayout operation
    fileprivate var _willLayout: Bool = false;
    
    /// Events
    fileprivate var _onClick: (() -> ())?;
    
    /// The layer to draw on, if there is one
    fileprivate var _drawingLayer: FloCanvasLayer?;
    
    override init() {
        _bounds = Bounds(
            x1: Position.Start,
            y1: Position.Start,
            x2: Position.End,
            y2: Position.End
        );
        _subviews = [];

        _view = FloEmptyView.init();

        super.init();

        weak var this = self;
        
        _view.performLayout = { size in if let this = this { this.performLayout(size) } };
        _view.onClick       = { if let onClick = this?._onClick { onClick(); return true; } else { return false; } }
    }
    
    required init(withView view: FloContainerView) {
        _bounds = Bounds(
            x1: Position.Start,
            y1: Position.Start,
            x2: Position.End,
            y2: Position.End
        );
        _subviews = [];
        
        _view = view;
        
        super.init();
        
        weak var this = self;
        
        _view.performLayout = { size in if let this = this { this.performLayout(size) } };
        _view.onClick       = { if let onClick = this?._onClick { onClick(); return true; } else { return false; } }
    }
    
    convenience init(withControl: NSControl) {
        self.init();
        
        _control = withControl;
        _view.addContainerSubview(control);
    }
    
    ///
    /// The bounds of this view
    ///
    internal var floBounds: Bounds {
        get { return _bounds; }
    }

    ///
    /// The view that this is managing
    ///
    public var view: NSView! {
        get { return _view.asView; }
    }
    
    ///
    /// The subviews that should be laid out within this view
    ///
    public var layoutSubviews: [FloView] {
        get {
            return _subviews;
        }
    }
    
    /*
    ///
    /// The bounds within which the subviews should be laid out
    ///
    public var layoutBounds: NSRect {
        get {
            return _view.asView.bounds;
        }
    }
    */
    
    public var control: NSControl {
        get {
            if let control = _control {
                // Use the existing control if there is one
                return control;
            } else {
                // Default control is a label
                let label   = NSTextField.init(labelWithString: "");
                label.font  = NSFontManager.shared.font(withFamily: "Lato", traits: NSFontTraitMask(), weight: 5, size: 13.0);
                
                view.addSubview(label);
                _control = label;

                return label;
            }
        }
    }
    
    ///
    /// Performs layout of this view immediately
    ///
    public func performLayout(_ size: NSSize) {
        // Just pass the request on to the layout class
        Layout.layoutView(view: self, size: size);
    }
    
    ///
    /// Invalidates the layout of this view
    ///
    public func invalidateLayout() {
        if !_willLayout {
            _willLayout = true;
            
            RunLoop.main.perform(inModes: [RunLoop.Mode.default, RunLoop.Mode.eventTracking], block: {
                self._willLayout = false;
                self.performLayout(self._view.layoutSize);
            });
        }
    }
    
    ///
    /// Performs the click event/action for this view (callback for controls)
    ///
    @objc func onClick() {
        _view.triggerClick();
    }
    
    ///
    /// Sends an event if this view (or its control) is clicked
    ///
    @objc public func requestClick(_ events: FloEvents, withName: String?) {
        _onClick = { events.sendClick(withName); };
    }
    
    ///
    /// Sends an event if this view is scrolled
    ///
    @objc public func requestVirtualScroll(_ events: FloEvents, withName: String?, width scrollWidth: Float64, height scrollHeight: Float64) {
        var (x, y)          = (UInt32(0), UInt32(0));
        var (width, height) = (UInt32(0), UInt32(0));
        
        _view.onScroll = { visibleRect in
            let (newXf, newYf)          = (Float64(visibleRect.minX) / scrollWidth, Float64(visibleRect.minY) / scrollHeight);
            let (newXi, newYi)          = (UInt32(floor(Float64.maximum(newXf, 0))), UInt32(floor(Float64.maximum(newYf, 0))));
            
            let (newWidthf, newHeightf) = (Float64(visibleRect.width) / scrollWidth, Float64(visibleRect.height)/scrollHeight);
            let (newWidthi, newHeighti) = (UInt32(floor(newWidthf)+1.0), UInt32(floor(newHeightf)+1.0));
            
            if newXi != x || newYi != y || newWidthi != width || newHeighti != height {
                x       = newXi;
                y       = newYi;
                width   = newWidthi;
                height  = newHeighti;
                
                events.sendVirtualScroll(withName!, left: newXi, top: newYi, width: width, height: height);
            }
        };
    }
    
    ///
    /// Removes this view from its superview
    ///
    @objc public func viewRemoveFromSuperview() {
        // Remove the view from the view hierarchy
        _view.asView.removeFromSuperview();
        
        // Remove from its parent FloView
        if let superview = _superview {
            superview._subviews.removeAll(where: { view in return view == self });
        }
    }
    
    ///
    /// Adds a subview to this view
    ///
    @objc(viewAddSubView:) public func viewAddSubView(subview: FloView) {
        subview.viewRemoveFromSuperview();
        
        self._subviews.append(subview);
        subview._superview = self;
        
        if let subview = subview.view {
            _view.addContainerSubview(subview);
        }
        
        // View will need to be laid out again
        invalidateLayout();
    }
    
    ///
    /// Sets the position of a side of the view
    ///
    func set_side_position(_ side: Int32, _ position: Position) {
        switch (side) {
        case 0: _bounds.x1 = position;
        case 1: _bounds.y1 = position;
        case 2: _bounds.x2 = position;
        case 3: _bounds.y2 = position;
        default: break;
        }
    }
    
    @objc(viewSetSide:at:) public func viewSetSide(side: Int32, at: Float32) {
        set_side_position(side, Position.At(at));
    }

    @objc(viewSetSide:offset:) public func viewSetSide(side: Int32, offset: Float32) {
        set_side_position(side, Position.Offset(offset));
    }

    @objc(viewSetSide:stretch:) public func viewSetSide(side: Int32, stretch: Float32) {
        set_side_position(side, Position.Stretch(stretch));
    }

    @objc(viewSetSideAtStart:) public func viewSetSideAtStart(side: Int32) {
        set_side_position(side, Position.Start);
    }

    @objc(viewSetSideAtEnd:) public func viewSetSideAtEnd(side: Int32) {
        set_side_position(side, Position.End);
    }

    @objc(viewSetSideAfter:) public func viewSetSideAfter(side: Int32) {
        set_side_position(side, Position.After);
    }
    
    ///
    /// Sets the z-ordering of this view
    ///
    @objc public func viewSetZIndex(_ zIndex: Float64) {
        _view.asView.layer?.zPosition = CGFloat(zIndex);
    }
    
    ///
    /// Sets the foreground (text) colour of the view
    ///
    @objc public func viewSetForegroundRed(_ red: Float64, green: Float64, blue: Float64, alpha: Float64) {
        let col = NSColor(calibratedRed: CGFloat(red), green: CGFloat(green), blue: CGFloat(blue), alpha: CGFloat(alpha));
        
        // TODO: need to support attributed strings :-/
    }

    ///
    /// Sets the background colour of the view
    ///
    @objc public func viewSetBackgroundRed(_ red: Float64, green: Float64, blue: Float64, alpha: Float64) {
        let col = NSColor(calibratedRed: CGFloat(red), green: CGFloat(green), blue: CGFloat(blue), alpha: CGFloat(alpha));
        
        _view.asView.layer?.backgroundColor = col.cgColor;
    }
    
    var _text: FloProperty?;
    ///
    /// Sets the text for the view
    ///
    @objc public func viewSetText(_ text: FloProperty) {
        _text = text;
        
        text.trackValue({ value in
            if case let PropertyValue.String(value) = value {
                self.control.stringValue = value;
            }
        });
    }
    
    var _imageView: NSImageView!;
    
    ///
    /// Sets the image for the view
    ///
    @objc public func viewSetImage(_ image: NSImage) {
        // Add an image view to this view if one does not already exist
        if _imageView == nil {
            _imageView = NSImageView.init();
            _view.addContainerSubview(_imageView);
        }
        
        // Change its image
        _imageView!.image = image;
    }
    
    ///
    /// Sets the font size of the control for this view
    ///
    @objc public func viewSetFontSize(_ size: Float64) {
        let existingFont    = control.font!;
        let newFont         = NSFontManager.shared.convert(existingFont, toSize: CGFloat(size));
        
        control.font        = newFont;
    }
    
    ///
    /// Converts a weight from a value like 100, 200, 400, etc to a font manager weight (0-15)
    ///
    func convertWeight(_ weight: Float64) -> Int {
        if weight <= 150.0 {
            return 1;
        } else if weight <= 450.0 {
            return 5;
        } else if weight <= 750.0 {
            return 7;
        } else {
            return 10;
        }
    }
    
    ///
    /// Sets the font weight of the control for this view
    ///
    @objc public func viewSetFontWeight(_ weight: Float64) {
        let existingFont        = control.font!;
        let fontManagerWeight   = convertWeight(weight);
        let family              = existingFont.familyName!;
        let size                = existingFont.pointSize;
        let traits              = NSFontTraitMask();
        
        let newFont             = NSFontManager.shared.font(withFamily: family, traits: traits, weight: fontManagerWeight, size: size);
        
        control.font        = newFont;
    }

    ///
    /// Sets the text alignment of the control for this view
    ///
    /// Alignments are:
    ///     0 - Left
    ///     1 - Center
    ///     2 - Right
    ///
    @objc public func viewSetTextAlignment(_ alignment: UInt32) {
        switch alignment {
        case 0:     control.alignment = NSTextAlignment.left;   break;
        case 1:     control.alignment = NSTextAlignment.center; break;
        case 2:     control.alignment = NSTextAlignment.right;  break;
        default:    break;
        }
    }
    
    ///
    /// Sets the minimum size for the scrollable area of this view
    ///
    @objc(viewSetScrollMinimumSizeWithWidth:height:) public func viewSetScrollMinimumSize(withWidth width: Float64, height: Float64) {
        _view.scrollMinimumSize = (width, height);
    }
    
    func getScrollBarVisibility(_ intVisibility: UInt32) -> ScrollBarVisibility {
        switch (intVisibility) {
        case 0:     return ScrollBarVisibility.Never;
        case 1:     return ScrollBarVisibility.Always;
        case 2:     return ScrollBarVisibility.OnlyIfNeeded;
        default:    return ScrollBarVisibility.OnlyIfNeeded;
        }
    }
    
    ///
    /// Sets the horizontal scroll bar visibility
    ///
    @objc public func viewSetHorizontalScrollVisibility(_ visibility: UInt32) {
        let (_, vert) = _view.scrollBarVisibility;
        _view.scrollBarVisibility = (getScrollBarVisibility(visibility), vert);
    }

    ///
    /// Sets the horizontal scroll bar visibility
    ///
    @objc public func viewSetVerticalScrollVisibility(_ visibility: UInt32) {
        let (horiz, _) = _view.scrollBarVisibility;
        _view.scrollBarVisibility = (horiz, getScrollBarVisibility(visibility));
    }
    
    ///
    /// Updates the bounds of the drawing layer (and its context) after the
    ///
    func drawingLayerBoundsChanged(_ newBounds: ContainerBounds) {
        autoreleasepool {
            let layer = _drawingLayer!;
            
            // Work out the screen resolution of the current window
            var resolutionMultiplier = CGFloat(1.0);
            if let window = _view.asView.window {
                if let screen = window.screen {
                    resolutionMultiplier = screen.backingScaleFactor;
                }
            }
            
            // Perform the action instantly rather than with the default animation
            CATransaction.begin();
            CATransaction.setAnimationDuration(0.0);
            CATransaction.setDisableActions(true);

            // Move the layer so that it fills the visible bounds of the view
            let parentBounds    = _view.asView.layer!.bounds;
            var visibleRect     = newBounds.visibleRect;
            
            visibleRect.origin.x += parentBounds.origin.x;
            visibleRect.origin.y += parentBounds.origin.y;
            if visibleRect.size.width < 1.0 { visibleRect.size.width = 1.0; }
            if visibleRect.size.height < 1.0 { visibleRect.size.height = 1.0; }
            
            layer.frame         = visibleRect;
            
            CATransaction.commit();
            
            // Regenerate the graphics context so that it's the appropriate size for the layer
            if _drawingLayer?._visibleRect.size != newBounds.visibleRect.size {
                // Backing will have changed size, so invalidate it entirely
                _drawingLayer?.invalidateAllLayers();
            } else {
                // Just trigger a redraw
                _drawingLayer?._triggerRedraw?(newBounds.totalSize, newBounds.visibleRect);
            }
            
            _drawingLayer?._canvasSize      = newBounds.totalSize;
            _drawingLayer?._visibleRect     = newBounds.visibleRect;
            
            // Set the initial transformation of the context
            _drawingLayer?._resolution      = resolutionMultiplier;
            _drawingLayer?.contentsScale    = resolutionMultiplier;
            
            _drawingLayer?.setNeedsDisplay();
        }
    }
    
    ///
    /// Creates the layer that will be used to draw canvas items for this view
    ///
    func createCanvasDrawingLayer(_ events: FloEvents) {
        // Create the layer
        let layer       = FloCanvasLayer();
        
        // Layer should not animate its contents
        layer.actions = [
            "onOrderIn":    NSNull(),
            "onOrderOut":   NSNull(),
            "sublayers":    NSNull(),
            "contents":     NSNull(),
            "bounds":       NSNull(),
            "frame":        NSNull()
        ];
        
        _drawingLayer = layer;
        
        // Reset the layer size when the bounds change
        weak var this = self;
        var willChangeBounds = false;
        _view.boundsChanged = { newBounds in
            if !willChangeBounds {
                willChangeBounds = true;
                
                RunLoop.main.perform(inModes: [RunLoop.Mode.default, RunLoop.Mode.eventTracking], block: {
                    willChangeBounds = false;
                    if let this = this {
                        // Update the layer bounds
                        this.drawingLayerBoundsChanged(newBounds);
                    }
                });
            }
        }
        
        var initialSize = _view.layoutSize;
        if initialSize.width < 1 { initialSize.width = 1 }
        if initialSize.height < 1 { initialSize.height = 1 }
        
        layer._triggerRedraw        = { (canvasSize, viewport) in events.redrawCanvas(with: canvasSize, viewport: viewport); }
        layer.backgroundColor       = CGColor.clear;
        layer.frame                 = CGRect(x: 0, y: 0, width: initialSize.width, height: initialSize.height);
        layer.drawsAsynchronously  = false;
        layer.setNeedsDisplay();
        
        RunLoop.main.perform(inModes: [RunLoop.Mode.default, RunLoop.Mode.eventTracking], block: { self._view.setCanvasLayer(layer) });
    }
    
    ///
    /// Retrieves the drawing context for this view
    ///
    @objc public func viewGetCanvasForDrawing(_ events: FloEvents, layer: UInt32) -> CGContext? {
        // Create the drawing layer if one doesn't exist yet
        if _drawingLayer == nil {
            createCanvasDrawingLayer(events);
        }
        
        // Make sure the backing for the layer has been created
        return _drawingLayer?.getContextForLayer(id: layer);
    }
    
    var _willUpdateCanvas = false;
    ///
    /// Drawing on the context has finished
    ///
    @objc public func viewFinishedDrawing() {
        _drawingLayer?.setNeedsDisplay();
    }
    
    ///
    /// The drawing canvas should be entirely cleared
    ///
    @objc public func viewClearCanvas() {
        _drawingLayer?.clearBackingLayers();
    }
}
