//
//  FloScrollingView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 06/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

public class FloScrollingView : NSScrollView, FloContainerView {
    public required init?(coder: NSCoder) {
        _scrollMinimumSize = (0,0);
        _scrollBarVisibility = (ScrollBarVisibility.OnlyIfNeeded, ScrollBarVisibility.OnlyIfNeeded);
        
        super.init(coder: coder)

        self.documentView = FloEmptyView.init(frame: NSRect(x: 0, y: 0, width: 4000, height: 4000));
        self.documentView?.wantsLayer = false;

        self.wantsLayer             = true;
        self.hasHorizontalScroller  = true;
        self.hasVerticalScroller    = true;
        self.autohidesScrollers     = true;
        
        self.contentView.postsBoundsChangedNotifications = true;
        NotificationCenter.default.addObserver(self, selector: #selector(triggerOnScroll), name: NSView.boundsDidChangeNotification, object: self.contentView);
    }
    
    required public override init(frame frameRect: NSRect) {
        _scrollMinimumSize = (0,0);
        _scrollBarVisibility = (ScrollBarVisibility.OnlyIfNeeded, ScrollBarVisibility.OnlyIfNeeded);

        super.init(frame: frameRect);

        self.documentView = FloEmptyView.init(frame: NSRect(x: 0, y: 0, width: 4000, height: 4000));
        self.documentView?.wantsLayer = false;

        self.wantsLayer             = true;
        self.hasHorizontalScroller  = true;
        self.hasVerticalScroller    = true;
        self.autohidesScrollers     = true;
        
        self.contentView.postsBoundsChangedNotifications = true;
        NotificationCenter.default.addObserver(self, selector: #selector(triggerOnScroll), name: NSView.boundsDidChangeNotification, object: self.contentView);
    }
    
    override public var isOpaque: Bool { get { return false } }

    ///
    /// Adds a subview to this container
    ///
    func addContainerSubview(_ subview: NSView) {
        // Add to the document view
        self.documentView!.addSubview(subview);
    }
    
    /// The views that are fixed relative to this view (and where they are fixed, and their original bounds)
    fileprivate var _fixedViews: [(NSView, FixedAxis, NSRect)] = [];
    
    ///
    /// Moves the fixed views so they're visible relative to this scroll view
    ///
    func repositionFixedViews() {
        // Nothing to do if there are fixed views
        if _fixedViews.count == 0 {
            return;
        }
        
        // We update the frame to be relative to the visible rect
        // (NSScrollView also has a 'floating subview' mechanism, which we're not using because we can't quite get the behaviour we want)
        let visible = self.documentView!.visibleRect;
        
        // Disable any positioning animation
        CATransaction.begin();
        CATransaction.setAnimationDuration(0.0);
        CATransaction.disableActions();
        
        // Iterate through the views
        for (view, axis, originalFrame) in _fixedViews {
            // Work out the new frame for this view relative to the visible area
            var newFrame = originalFrame;
            
            switch (axis) {
            case .None:         break;
            case .Horizontal:   newFrame.origin.x += visible.origin.x;
            case .Vertical:     newFrame.origin.y += visible.origin.y;
            case .Both:         newFrame.origin.x += visible.origin.x; newFrame.origin.y += visible.origin.y; break;
            }
            
            // Reposition the view
            view.setFrameOrigin(newFrame.origin);
        }
        
        CATransaction.commit();
    }
    
    ///
    /// Sets the sizing for the document view
    ///
    func layoutDocumentView() {
        // Decide on the size of the document view
        let (minX, minY)    = scrollMinimumSize;
        let contentSize     = contentView.bounds.size;
        
        let sizeX           = CGFloat.maximum(CGFloat(minX), contentSize.width);
        let sizeY           = CGFloat.maximum(CGFloat(minY), contentSize.height);
        
        let newSize         = CGSize(width: sizeX, height: sizeY);
        
        documentView?.setFrameSize(newSize);
        
        // Perform general layout
        self.performLayout?(newSize);
        
        // Check for any fixed views
        _fixedViews = [];
        for subview in documentView!.subviews {
            if let containerView = subview as? FloContainerView {
                if containerView.viewState.fixedAxis != FixedAxis.None {
                    // The frame indicates where the view is fixed relative to this one
                    _fixedViews.append((subview, containerView.viewState.fixedAxis, subview.frame));
                }
            }
        }
        
        // Set the initial position of the fixed views
        repositionFixedViews();

        // Any subviews that are not themselves FloContainers are sized to fill this view
        for subview in documentView!.subviews {
            if (subview as? FloContainerView) == nil {
                subview.frame = NSRect(origin: CGPoint(x: 0, y: 0), size: newSize);
            }
        }
    }

    /// The size of the layout area for this view
    var layoutSize : NSSize {
        get {
            if let documentView = documentView {
                return documentView.bounds.size;
            } else {
                let (width, height) = scrollMinimumSize;
                return NSSize(width: width, height: height);
            }
        }
    };

    ///
    /// Containers cause the layout algorithm to run when they are resized
    ///
    override public func setFrameSize(_ newSize: NSSize) {
        super.setFrameSize(newSize);
        
        layoutDocumentView();
        triggerOnScroll();
    }

    fileprivate var _scrollMinimumSize: (Float64, Float64);

    /// The minimum size of the scroll area for this view
    var scrollMinimumSize: (Float64, Float64) {
        get { return _scrollMinimumSize; }
        set(value) {
            _scrollMinimumSize = value;
        }
    }

    fileprivate var _scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility);

    /// The visibility of the horizontal and vertical scroll bars
    var scrollBarVisibility: (ScrollBarVisibility, ScrollBarVisibility) {
        get { return _scrollBarVisibility; }
        set(value) {
            _scrollBarVisibility = value;
            
            // Set the scrollbar visibility
            let (horiz, vert) = value;
            switch (horiz) {
            case ScrollBarVisibility.Always, ScrollBarVisibility.OnlyIfNeeded:  self.hasHorizontalScroller = true; break;
            case ScrollBarVisibility.Never:                                     self.hasHorizontalScroller = false; break;
            }

            switch (vert) {
            case ScrollBarVisibility.Always, ScrollBarVisibility.OnlyIfNeeded:  self.hasVerticalScroller = true; break;
            case ScrollBarVisibility.Never:                                     self.hasVerticalScroller = false; break;
            }

            // Cocoa can't auto-hide individually, so we always auto-hide both scrollbars
            switch (value) {
            case (ScrollBarVisibility.OnlyIfNeeded, _), (_, ScrollBarVisibility.OnlyIfNeeded):
                self.autohidesScrollers = true;
                break;
            
            default:
                self.autohidesScrollers = false;
                break;
            }
        }
    }

    /// Stores the general state of this view
    var viewState : ViewState = ViewState();

    /// The FloView that owns this container view (should be a weak reference)
    weak var floView: FloView?;

    /// Returns this view as an NSView
    var asView : NSView { get { return self; } };
    
    /// Event handler: user clicked in the view
    var onClick: (() -> Bool)?;

    /// Event handler: value has changed
    var onEditValue: ((PropertyValue) -> ())?;
    
    /// Event handler: value has been set
    var onSetValue: ((PropertyValue) -> ())?;
    
    /// Event handler: control has obtained keyboard focus
    var onFocused: (() -> ())?;

    /// Event handler: user has dragged this control
    var onDrag: ((DragAction, CGPoint, CGPoint) -> ())?;

    /// Event handlers when particular devices are used for painting actions
    var onPaint: [FloPaintDevice: (FloPaintStage, AppPainting) -> ()] = [FloPaintDevice: (FloPaintStage, AppPainting) -> ()]();

    /// Event handler: user scrolled/resized so that a particular region is visible
    var _onScroll: ((NSRect) -> ())?;
    var onScroll: ((NSRect) -> ())? {
        get { return _onScroll; }
        set(value) {
            _onScroll = value;
            
            triggerOnScroll();
        }
    }

    /// The affine transform for the canvas layer
    var canvasAffineTransform: CGAffineTransform?;

    /// Event handler: user performed layout on this view
    var performLayout: ((NSSize) -> ())?;
    
    /// Event handler: The bounds of the container have changed
    var boundsChanged: ((ContainerBounds) -> ())?;

    /// Triggers the click event for this view
    func triggerClick() {
        bubbleUpEvent(source: self, event_handler: { (container) in
            if let onClick = container.onClick {
                return onClick();
            } else {
                return false;
            }
        });
    }
    
    /// Triggers the scroll event for this view
    @objc func triggerOnScroll() {
        // Make sure the fixed views are visible
        repositionFixedViews();
        
        // This also changes the bounds of the view
        triggerBoundsChanged();
        
        // Find the area that's visible on screen
        let visibleRect = self.convert(bounds, to: documentView);
        
        // Send the onScroll event
        _onScroll?(visibleRect);
    }

    /// Sets the layer displayed for the canvas
    func setCanvasLayer(_ layer: CALayer) {
        self.documentView!.layer!.addSublayer(layer);
    }

    ///
    /// Triggers the bounds changed event for this view
    ///
    func triggerBoundsChanged() {
        // For scrolling views, we actually trigger for all the subviews of the document view
        var toProcess = [self.documentView!];
        
        while let view = toProcess.popLast() {
            // If the view is a container view, trigger its bounds changed event
            if let view = view as? FloContainerView {
                view.triggerBoundsChanged();
            }
            
            // If the view is not a scrolling view, add its subviews (nested scrolling views will already have triggered the event)
            if !(view is FloScrollingView) {
                for subview in view.subviews {
                    toProcess.append(subview);
                }
            }
        }
    }
    
    /// Sets the text label for this view
    func setTextLabel(label: String) {
        // Scroll view just acts as a container, can't have a label
    }

    /// Sets the font size for this view
    func setFontSize(points: Float64) {
    }
    
    /// Sets the font weight for this view
    func setFontWeight(weight: Float64) {
    }
    
    /// Sets the text alignment for this view
    func setTextAlignment(alignment: NSTextAlignment) {
    }

    /// Sets the foreground colour of the control
    func setForegroundColor(color: NSColor) {
        
    }

    /// Sets part of the state of this control
    func setState(selector: ViewStateSelector, toProperty: FloProperty) {
        viewState.retainProperty(selector: selector, property: toProperty);
    }
}
