import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {createBuildStarterBase} from "@/factorio-bot/tasks/build-starter-base-task";
import {Entities, InventoryType, RequestEntity, Technologies} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {FactorioApi} from "@/factorio-bot/restApi";
import {missingIngredients, sleep} from "@/factorio-bot/util";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";
import {createLoopResearchTask} from "@/factorio-bot/tasks/loop-research-task";
import {createBuildStarterLabTask} from "@/factorio-bot/tasks/build-starter-lab-task";

const TASK_TYPE = 'research'

type TaskData = {
    name: string,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        return
    }
    // sort by already has correct item
    // bots.sort(sortBotsByInventory([minerName, furnaceName]))
    const bot = bots[0]
    const tech = store.state.force.technologies[data.name]
    if (!tech) {
        throw new Error(`tech not found: ${data.name}`)
    }
    if (tech.researched) {
        return
    }
    const addAndExecuteSubtask = async (subtask: Task): Promise<void> => {
        store.commit('addSubTask', {id: task.id, task: subtask})
        await executeTask(store, bots, subtask)
    }
    if (!store.state.world.starterScienceBlueprints) {
        await addAndExecuteSubtask(await createBuildStarterBase(store, 6, 4, 3, 6, 1))
    }
    if (!store.state.world.starterScienceBlueprints || store.state.world.starterScienceBlueprints.length === 0) {
        throw new Error("should have one lab?")
    }
    if (!store.state.world.starterSteamEngineBlueprints || store.state.world.starterSteamEngineBlueprints.length === 0) {
        throw new Error("should have one lab?")
    }

    if (tech.prerequisites && tech.prerequisites.length > 0) {
        for (const name of tech.prerequisites) {
            const requirement = store.state.force.technologies[name]
            if (!requirement.researched) {
                await addAndExecuteSubtask(await createResearchTask(store, name))
            }
        }
    }

    // prioritize automation research
    if (!store.state.force.technologies[Technologies.automation].researched && data.name !== Technologies.automation) {
        await addAndExecuteSubtask(await createResearchTask(store, Technologies.automation))
    }
    // prioritize logistics research
    if (!store.state.force.technologies[Technologies.logistics].researched && data.name !== Technologies.logistics && data.name !== Technologies.automation) {
        await addAndExecuteSubtask(await createResearchTask(store, Technologies.logistics))
    }

    await FactorioApi.addResearch(data.name)
    if (store.state.force.technologies[Technologies.automation].researched) {
        const scienceGhosts = store.state.world.starterScienceBlueprints.flatMap(entities => entities.filter(entity => entity.ghostName))

        if (scienceGhosts.length > 0) {
             // finish science
            await addAndExecuteSubtask(await createCraftTask(store, Entities.assemblingMachine1, 4, false))
            await addAndExecuteSubtask(await createCraftTask(store, Entities.inserter, 4, false))
            await addAndExecuteSubtask(await createCraftTask(store, Entities.smallElectricPole, 2, false))
            for (const scienceGhost of scienceGhosts) {
                const realEntity = await bot.reviveGhost(scienceGhost)
                store.commit('updateScienceGhost', realEntity)
            }
        }
        // gather needed iron-plates and copper-plates
        for (const ingredient of tech.researchUnitIngredients) {
            if (ingredient.name !== Entities.automationSciencePack) {
                throw new Error("only up to automation-science-pack yet")
            }
            if (bot.mainInventory(ingredient.name) < tech.researchUnitCount) {
                const ingredients = missingIngredients(store.state.recipes, {}, ingredient.name, tech.researchUnitCount);
                for(const name of Object.keys(ingredients)) {
                    const subtask = await createGatherTask(store, name, ingredients[name])
                    store.commit('addSubTask', {id: task.id, task: subtask})
                    store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
                    await executeTask(store, bots, subtask)
                    store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));

                    if (name === Entities.ironPlate) {
                        const assemblingMachines = store.state.world.starterScienceBlueprints.flatMap(blueprint => blueprint.filter(entity => entity.recipe === Entities.ironGearWheel))
                        if (assemblingMachines.length === 0) {
                            throw new Error("no assembly machines found for iron?")
                        }
                        for(const machine of assemblingMachines) {
                            await bot.insertToInventory(machine.name, machine.position, InventoryType.furnace_source, name, Math.ceil(ingredients[name] / assemblingMachines.length))
                        }
                    } else if(name === Entities.copperPlate) {
                        const assemblingMachines = store.state.world.starterScienceBlueprints.flatMap(blueprint => blueprint.filter(entity => entity.recipe === Entities.automationSciencePack))
                        if (assemblingMachines.length === 0) {
                            throw new Error("no assembly machines found for copper?")
                        }
                        for(const machine of assemblingMachines) {
                            await bot.insertToInventory(machine.name, machine.position, InventoryType.furnace_source, name, Math.ceil(ingredients[name] / assemblingMachines.length))
                        }
                    } else {
                        throw new Error(`unsupported ingredient: ${name}`)
                    }
                }
            }
        }

        if (store.state.world.starterScienceBlueprints.length < 3) {
            const subtask = await createBuildStarterLabTask(store, 1, false)
            store.commit('addSubTask', {id: task.id, task: subtask})
            store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
            await executeTask(store, bots, subtask)
        }
    } else {
        for (const ingredient of tech.researchUnitIngredients) {
            if (bot.mainInventory(ingredient.name) < tech.researchUnitCount) {
                const subtask = await createCraftTask(store, ingredient.name, tech.researchUnitCount - bot.mainInventory(ingredient.name), true)
                store.commit('addSubTask', {id: task.id, task: subtask})
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
                await executeTask(store, bots, subtask)
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
            }
        }
        const boilers = store.state.world.starterSteamEngineBlueprints[0].filter(entity => entity.name === Entities.boiler)
        if (boilers.length === 0) {
            throw new Error("could not find boiler?")
        }
        const boilerInventories = await FactorioApi.inventoryContentsAt(boilers.flatMap(entity => {
            const requestEntities: RequestEntity[] = [];
            requestEntities.push({
                position: entity.position,
                name: entity.name
            })
            return requestEntities
        }))
        for(const boiler of boilerInventories) {
            const fuel = (boiler.fuelInventory || {})[Entities.coal] || 0
            if (fuel < 10) {
                const toInsert = Math.min(bot.mainInventory(Entities.coal), 10 - fuel)
                if (toInsert > 0) {
                    await bot.insertToInventory(
                        Entities.boiler,
                        boiler.position,
                        InventoryType.chest_or_fuel,
                        Entities.coal,
                        toInsert
                    );
                }
            }
        }
        const starterLabs = store.state.world.starterScienceBlueprints.flatMap(blueprintEntities => blueprintEntities
            .filter(entity => entity.name === Entities.lab));

        for (const ingredient of tech.researchUnitIngredients) {
            for (let i=0; i<tech.researchUnitCount * ingredient.amount; i++) {

                const labIndex = i % starterLabs.length
                const lab = starterLabs[labIndex]

                const subtask = await createCraftTask(store, ingredient.name, 1, false)
                store.commit('addSubTask', {id: task.id, task: subtask})
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
                await executeTask(store, bots, subtask)
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
                await bot.insertToInventory(
                    Entities.lab,
                    lab.position,
                    InventoryType.furnace_source,
                    ingredient.name,
                    1
                );
            }
        }
    }
    const subtask = await createLoopResearchTask(store, Entities.coal, data.name)
    store.commit('addSubTask', {id: task.id, task: subtask})
    store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
    await executeTask(store, bots, subtask)
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createResearchTask(store: Store<State>, name: string): Promise<Task> {
    const data: TaskData = {
        name,
    }
    return createTask(TASK_TYPE, `Research ${name}`, data)
}