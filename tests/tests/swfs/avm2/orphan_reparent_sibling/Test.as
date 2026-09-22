package {
    import flash.display.MovieClip;
    import flash.display.Sprite;
    import flash.events.Event;

    // Frame 2 removes "clip". Its children "first" and "second" have three frames.
    public class Test extends MovieClip {
        public var clip:MovieClip;
        private var kept:MovieClip;
        private var target:Sprite = new Sprite();
        private var tick:int = 0;

        public function Test() {
            stop();
            kept = clip;
            clip.stop();
            clip.first.stop();
            clip.second.stop();
            clip.first.addFrameScript(1, function():void {
                trace("reparent second");
                target.addChild(kept.second);
            });
            clip.second.addFrameScript(1, function():void {
                trace("second frame 2");
            });
            addEventListener(Event.ENTER_FRAME, function(e:Event):void {
                if (++tick == 1) {
                    gotoAndStop(2);
                } else if (tick == 2) {
                    kept.first.play();
                    kept.second.play();
                }
            });
            addEventListener(Event.EXIT_FRAME, function(e:Event):void {
                if (tick == 3 || tick == 4) {
                    trace("exit", tick);
                }
            });
        }
    }
}
