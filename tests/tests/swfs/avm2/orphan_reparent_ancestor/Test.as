package {
    import flash.display.MovieClip;
    import flash.display.Sprite;
    import flash.events.Event;

    // Frame 2 removes "clip". Its children have three frames. Frame 2 of
    // "first" constructs MoveAncestor; frame 2 of "second" constructs Observer.
    public class Test extends MovieClip {
        public static var instance:Test;
        public var clip:MovieClip;
        public var kept:MovieClip;
        public var target:Sprite = new Sprite();
        private var tick:int = 0;

        public function Test() {
            instance = this;
            stop();
            kept = clip;
            clip.stop();
            clip.first.stop();
            clip.second.stop();
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

    public class MoveAncestor extends MovieClip {
        public function MoveAncestor() {
            trace("reparent ancestor");
            Test.instance.target.addChild(Test.instance.kept);
        }
    }

    public class Observer extends MovieClip {
        public function Observer() {
            trace("second constructor");
        }
    }
}
