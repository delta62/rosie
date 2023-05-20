function main(args)
    print(string.format("App invoked with the phrase '%s'", args.phrase))

    api.delayed(1000, { message = "it's been one second!" })
end
