package {
    import flash.display.*;
    import flash.filters.GradientBevelFilter;
    import flash.geom.Point;

    [SWF(width="480", height="280", frameRate="30", backgroundColor="#ffffff")]
    public class Test extends Sprite {
        public function Test() {
            stage.scaleMode = StageScaleMode.NO_SCALE;
            stage.align = StageAlign.TOP_LEFT;
            stage.quality = StageQuality.LOW;
            var types:Array = ["inner", "outer", "full"];
            for (var i:int = 0; i < 6; i++) {
                var shape:Sprite = makeShape();
                shape.x = 24 + (i % 3) * 155;
                shape.y = 24 + int(i / 3) * 88;
                shape.filters = [new GradientBevelFilter(5, 45,
                    [0xFF0000, 0, 0x0000FF], [1, 0, 1], [0, 128, 255],
                    8, 8, 1, 1, types[i % 3], i >= 3)];
                addChild(shape);
            }

            // A nontransparent midpoint makes the filter's output bounds visible.
            shape = makeShape();
            shape.x = 24;
            shape.y = 205;
            shape.filters = [new GradientBevelFilter(5, 45,
                [0xFF9900], [0.6], [96], 8, 8, 1, 1, "full", true)];
            addChild(shape);

            // Multiple colors/alpha stops, with the same knockout shading used by a timer bar.
            shape = makeShape();
            shape.x = 179;
            shape.y = 205;
            shape.filters = [new GradientBevelFilter(10, 248,
                [0xE0DFE3, 0xFFFFCC, 0xFFFFFF, 0x330000], [0, 143/255, 0, 1],
                [26, 61, 128, 255], 15, 15, 0.796875, 1, "inner", true)];
            addChild(shape);

            // The bitmap path must use the same filter implementation.
            var source:BitmapData = new BitmapData(145, 78, true, 0);
            shape = makeShape();
            shape.x = 16;
            shape.y = 16;
            var container:Sprite = new Sprite();
            container.addChild(shape);
            source.draw(container);
            var dest:BitmapData = new BitmapData(145, 78, true, 0);
            dest.applyFilter(source, source.rect, new Point(),
                new GradientBevelFilter(5, 0, [0xFF0000, 0, 0x0000FF],
                    [1, 0, 1], [0, 128, 255], 8, 8, 1, 1, "full", true));
            var bitmap:Bitmap = new Bitmap(dest);
            bitmap.x = 318;
            bitmap.y = 189;
            addChild(bitmap);
        }

        private function makeShape():Sprite {
            var shape:Sprite = new Sprite();
            shape.graphics.beginFill(0x6699CC);
            shape.graphics.drawRect(0, 0, 48, 48);
            shape.graphics.endFill();
            shape.graphics.beginFill(0xCC6699, 0.4);
            shape.graphics.drawRect(64, 0, 48, 48);
            shape.graphics.endFill();
            return shape;
        }
    }
}
