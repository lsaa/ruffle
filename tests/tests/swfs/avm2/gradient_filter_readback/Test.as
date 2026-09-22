package {
    import flash.display.Sprite;
    import flash.filters.GradientBevelFilter;
    import flash.filters.GradientGlowFilter;

    public class Test extends Sprite {
        public function Test() {
            for each (var filterClass:Class in [GradientBevelFilter, GradientGlowFilter]) {
                trace(filterClass);
                filters = [new filterClass(4, 45,
                    [0x123456, 0xABCDEF, 0x654321], [0, 0.2, 1], [17, 128, 239])];

                // Read back through DisplayObject.filters to exercise the native conversion.
                var filter:* = filters[0];
                trace("colors: " + filter.colors);
                var alphaBytes:Array = [];
                for each (var alpha:Number in filter.alphas) {
                    alphaBytes.push(Math.round(alpha * 255));
                }
                trace("alpha bytes: " + alphaBytes);
                trace("ratios: " + filter.ratios);
            }
        }
    }
}
