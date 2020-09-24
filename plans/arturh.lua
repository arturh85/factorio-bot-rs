-- globals:
-- all_bots: Vec<u32>
-- world: LuaFactorioWorld
-- rcon: LuaFactorioRcon
-- plan: LuaPlanner


blueprints = {
    StarterSteamEngineBoiler = "0eNqdkdEKwjAMRf8lz504nRv0V0Rkm0ECbVrWThxj/242RQXrgz6VhHtPLr0jNKZH3xFH0CNQ6ziA3o8Q6My1mXdx8AgaKKIFBVzbeQoRa5shn4kRJgXEJ7yCzqeDAuRIkfDOWYbhyL1tsBNBmqDAuyAmx/NFAWUiHOQppkl9QDYviK2NydBgGztqM+9MgvVAlSnU9rc8eYpR/BMnSdo9SY0jI5tvOYrVLuUvn35P/utpsUpLS5/6rX4FF+zCIq6qbZ5X1brcyP/fAHsdtKc=",
    StarterScience = "0eNqdlNtuwjAMht/F1w0iPUJfg8tpmtJglWg5VEnYhlDffUlBCKa2jF66tb/f9q/4DI08YmeF9lCfwWnWEW9Ia8U+xj9QlwmcoKa0T0Bwox3UbyFPtJrJmOFPHUINwqOCBDRTMWLOoWqk0C1RjB+ERkIhAvQeAzKwniIka+4q0v49AdReeIGXDobg9KGPqkEbkPPaCXTGhWKjr1ORvFwVw2TZqgg6Frm4dGGNJi0yS74PiBJiq3+00pe1ssVa2U1LaIfWh28j/PzKLwJ/BJL/C1LMQ4obxCkmJUGJ3FvBSWckzg09wSsXO1Y9bpEdvVEsZhLHBWqOpGP8c2yb1WLnlmtuXnJwO76s7UsOTkDoeqmFU0D68FxnLKN0gpA+I2QPhHAFhjNR352tJJaG1xOm8iyuZndxJPz4QusGWLqheZVvq7Ki67Io+/4XVduccQ==",
    FurnaceLine = "0eNqdm81vo0gQxf+ViLOd0NUf0D7ubTXSXuYwh9VoRJweL7sYEODsRlH+98WxZmKNaXiPk5UPfn5Vpqq7X+HX5LE6hbYr6yHZvSZPod93ZTuUTZ3sks/NqduHu7+Goe13Dw/fi/3QdGXz/t/9/b45PjyX4d+H7aemOv1h/37+cvg9DYeXfz59+5Jskr4u2u3QbA9d+XRm/5fsxG+Sl2Sn1NsmKR778bIhbM//15b1IdkN3SlsknLf1H2y+/M16ctDXVTna4eXNoyCyiEcR3JdHM8/DV1R923TDdvHUA3JyCzrpzC+zYhfvLgfmjpsv5+6utiHq2sFuHbftG3otm1VDNeXauDSsmvqmwvN29dNEuqhHMpwifz9h5dv9en4GLoxoFjMm6Rt+vLycb1nWN3b9xSn93bkP5Vd2F/++h7XL1ihsePr2wRIwyDN6DMwVhis/fg06j50w/i7G6CdD9jByiyjLIOxhsHmQMBuPmD/UTjHoqq2oRrfsCv327apwi0tm6epFA40YwJVeKE4iitABv1CzHiReEobXiU5xUXKRKULQeOFoqjWpfBS+akRAyPFovRC2GS5KJnnCV4viuqwQqwsVI8VpGKUWQgbLxlF9VnBa0ZRnVagolnojkIUDdUehSgaqj8KVDT5Qths0Sw0XE0UDdVxNVE0VMvVSNHI0l4MLxqhWq4mtmNUy9VI0chCi9R40QjVIjVeNEK1SI0UjSxsRDVZNLLQcg1eNEK1XIMXjVAt10BFs7C9NUTRUJ3RGPqMJRGJlibpCMlFTsG3h5cLZhKSoZBsBpKjED8D8SjkfDyIUWwKU/QMRcGUmdxagSkzybUapsxk1xqUInPZtTBlLrvwjStz2YXvXJnL7tWt21blMNl2fiyDiCnh6fK2vzYgM+VUpDTXRDwPBUQs0xG7KZ7QrtFNxJNcTfg7sVgNbfBYZDlwlrBiYtocbXJg2jLalMG4+TrvKBa+J7yYCCNLac8ECjVTtMeDcYXxYmJBa94ywdQZ3uTBwJbxYmJhO97iwNRlvCmDgfOV7lEsA57xYiKQPOUtEyjaXPEmDwYWxouJha15ywRTZ3iTBwNbxouJhe14iwNTl/GmDAbOV7pHsQx4xouJQHzKWyZQtF7xJg8GFsaLiYWtecsEU2d4kwcDW8aLiYXteIsDU5fxpgwGzle6R7EMeMaLMbHhYMqbMVC4KiXKJuPI+GFGco78UUyn+il0h64ZXyPsH03E3aA3P59HqNvT+WGJiXfivaoMOYKqlLeuHAZ2RG5kOTfNaYgmJ6NPqy52f+fEARVMhOcsvXxS2NVYfvl0iglTirMJI8Lklz5VFcc2fnKMJV5p4ugIxmc4BzMSHzVrB5U50haNSMuYMxgoLSe91og0D90VShZuC0mZ8xEWorBG8HSIQp1hQGmadJcj0gxzzgClWdKyjkhz2I3hl26MjDkDgCHmpJ8eCdEz+3RMmk5Jk35a2tV4HNhLg9KEdP4j0jSzPwWlGXKcEJHG75E8ps/R4BwDZ+Th4Ur2JI/ZFoGxe2JDgyGvBuSUmx2L2ihiTwJKpNxikKmZbQDINCt9zmguLbOQgyIdswKDzIxZOkHmWv8rmkzKAMNE2pRZtUAmtdyATFnpi8SSaamFBxTJH9cV+OSvpc+6KNnRj/Sj5Iye2KLknH4qHyV7euIKkompvyPJ/IgTJQs9lEXJmh9QomjDD1VRtOXniyh6xUwURWf8OA9F5/wIEkV7fhoHoolnDRRZi8TjBoosxkz4ASCKXvGAMope8Ygyirb86AlFO35chqJXTI5QdM5Pu1C05wdAU+ivm8u3L3dXX3LdJFUxosbf6fu734q+3N99Po7o8xdQN8lz6PrLxfm4Szc+c5lKnXVvb/8DoD9BzA==",
    MinerLine = "0eNqdl11v2yAUhv9KxLWdBPBH7MtN603Vq3ZSt2ma/MEyJAwIcFcr8n8vcapoWjztwJWFDQ+Hw3l5zQm1YmTacOlQfUI9s53h2nElUY0e1Wg6Vm9+Oadtvdv9bDqnDFdLd7vt1LB74ez3Ln24+/rhy2d83368e+0n3d4PzyhBVjY6dSo9Gt6f4a+oLhM0oZrgOUFNa5UYHUvP3TSXR1Q7M7IE8U5Ji+pvJ2T5UTbiPNRNmvmAuGODB8tmOLeYYJ0zvEsHLv34tDdcCOTRXPbMT4bn7wli0nHH2QW4NKYfchxaZnyH/6ASpJXll2Qs4eNtvizAP/00PTd+1PKVzMkNnVzpzjTSamVc2jLhbrH0Hbv/G5utYGlo0Pm/gi5W6Flw0AQSdB6MxRBsEbuBOWQDyyvdDo0Q6XUOrQS7ZZN3tl/BvEI7BKcgg6SgCsZSCBbvY8ssg5QZxsFhF6Cww0WXg7jRqitB6chiC/kAKWQcLr8DKCtFMLcEcSOVh/fr0sPh2sOgsxiHq6+CcEm0+m7jXqs3Eu98BGR94TLEIB8hNBwMchKSRWecgjKeR2cc5FUkXIoY5C+kDAeDHIZEiBLkASRclBhkAjRelQWkRmi8KitIjVASeayW68cqjRAjyFdo+F8oBhkLzaN3sALtYIQKQYZAw1VIVhzM34eWK1T9x60vQaLxJP/ugWw3nxojps2TUv79CzP2UkwHnJVZVRYl3hd5Mc9vau2sKw=="
}

function dump(tbl, label)
    print("dumping " .. label)
    for k,v in pairs(tbl) do
        print("- " .. tostring(k) .. ": " .. tostring(v))
    end
    print("------------")
end

--local recipe = world.recipe("inserter")
--print("recipe: " .. tostring(recipe))
--for k,v in pairs(recipe) do
--    print(k .. ": " .. tostring(v))
--end
--
--dumpPlayers()
function mine_with_bots(bots, search_center, name, entityName)
    --local nearest = rcon.find_nearest(search_center, 500, name, entityName, #bots)
--    local nearest = rcon.findByNameInRadius(name, "0,0", 500)
--    dump(nearest[1], "nearest")
    plan.groupStart("Mine with Bots")
    for idx,playerId in pairs(bots) do
--        local entity = nearest[idx]
        plan.mine(playerId, "0,0", name, 1)
    end
    plan.groupEnd()
end


mine_with_bots(all_bots, {0,0}, "rock-huge", nil)
mine_with_bots(all_bots, {0,0}, "rock-huge", nil)

--mine_with_bots(bots, {0,0}, nil, "tree")
--build_starter_base()
--research("automation")
--...
--start_rocket()

--function build_starter_base()
--    build_starter_miner_furnace("iron-ore", 2)
--    --    build_starter_miner_loop("coal", 2)
--    --    build_starter_miner_furnace("iron-ore", 2)
--    --    build_starter_miner_chest("stone", 2)
--    --    build_starter_steam_engine()
--    --    build_starter_science()
--end
--
--function build_starter_miner_furnace(ore_name, count)
--    local patches = resource_patches(ore_name)
--    local positions = patches.find_free_rect
--    for id, bot in pairs(bots) do
--
--    end
--end


