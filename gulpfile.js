
var gulp = require('gulp');
var minifyjs = require('gulp-js-minify');
var concat = require('gulp-concat');

gulp.task('default', function () {
    return gulp
        .src(['./JavaScript/HtmlDialog.js',
            './JavaScript/HtmlStaticElement.js',
            './JavaScript/Utils.js',
            './JavaScript/HtmlGraph.js',
            './JavaScript/HtmlStatusBar.js',
            './JavaScript/HtmlMain.js',
            './JavaScript/HtmlSubscribersGenerator.js',
            './JavaScript/main.js'])
        .pipe(minifyjs())
        .pipe(concat('app.js'))
        .pipe(gulp.dest('./wwwroot/js/'))
});