package {
    import flash.display.MovieClip;
    import flash.events.Event;

    // Frame 1 places "clip" (containing "child") and "direct".
    // Frame 2 removes both. All three MovieClips have three frames.
    public class Test extends MovieClip {
        public var clip:MovieClip;
        public var direct:MovieClip;
        private var ancestor:MovieClip;
        private var removed:MovieClip;
        private var tick:int = 0;

        public function Test() {
            stop();
            ancestor = clip;
            removed = direct;
            clip.stop();
            clip.child.stop();
            direct.stop();
            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                if (++tick == 1) {
                    ancestor.play();
                    removed.play();
                    gotoAndStop(2);
                } else if (tick == 2) {
                    ancestor.child.play();
                }
            });
            addEventListener(Event.EXIT_FRAME, function(e:Event):void {
                if (tick == 3 || tick == 4) {
                    trace("ancestor:", ancestor.currentFrame, "child:", ancestor.child.currentFrame,
                        "direct:", removed.currentFrame);
                }
            });
        }
    }
}
