module.exports = {
    plugins: {
        "@tailwindcss/postcss": {},
        "postcss-preset-env": {
            features: {
                "nesting-rules": true,
            },
            browsers: [
                "last 2 versions",
                "iOS >= 15",
                "Safari >= 15",
            ],
        },
    },
};
