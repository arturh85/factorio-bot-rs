import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, registerTaskRunner, Task, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {Entities, StarterMinerFurnace} from "@/factorio-bot/types";
import {buildBotQueueToCraft, processBotQueue} from "@/factorio-bot/bot-queue";
import {createPlaceStarterMinerFurnaceTask} from "@/factorio-bot/tasks/place-starter-miner-furnace";
import {FactorioApi} from "@/factorio-bot/restApi";
import {sortEntitiesByDistanceTo} from "@/factorio-bot/util";

const TASK_TYPE = 'build-starter-miner-furnace'
const minerName = Entities.burnerMiningDrill;
const furnaceName = Entities.stoneFurnace;

type TaskData = {
    oreName: string,
    plateName: string,
    minerSmelterCount: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<StarterMinerFurnace[]> {
    const data: TaskData = task.data as TaskData

    const firstBot = bots[0]
    // if this is the absolute beginning, first mine some huge rocks if found
    if (!store.state.world.starterMinerFurnaces) {
        const rockSearchAnchor = firstBot.player().position
        const results = await FactorioApi.findEntities(rockSearchAnchor, 500, Entities.rockHuge)
        if (results.length > 0) {
            store.commit('updateTask', updateTaskStatus(task, TaskStatus.WALKING));
            results.sort(sortEntitiesByDistanceTo(rockSearchAnchor));
            await (Promise.all(results.slice(0, bots.length).map((result, index) =>
                bots[index].mine(result.position, Entities.rockHuge, 1))))
            store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
        }
    }

    // each bot should first craft what it needs
    const queue = await buildBotQueueToCraft(store, task, bots, {
        [minerName]: data.minerSmelterCount,
        [furnaceName]: data.minerSmelterCount,
    })
    const excludePositions = Object.keys(store.state.players).map(key => store.state.players[key].position)
    const oreFieldTopLeft = await firstBot.findNearestRect(
        {x: 0, y: 0},
        data.oreName,
        2 * data.minerSmelterCount,
        4,
        excludePositions,
    );
    if (!oreFieldTopLeft) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.FAILED));
        throw new Error(`failed to find ore field for ${data.oreName}`)
    }
    const anchor = {...oreFieldTopLeft};
    anchor.x = Math.floor(anchor.x);
    anchor.y = Math.floor(anchor.y);
    const minerFurnaces: StarterMinerFurnace[] = []
    let toPlace = data.minerSmelterCount

    // each bot should place what it is responsible for
    for(const bot of bots) {
        const playerId = bot.playerId.toString()
        const botMinerSmelter = Math.min(
            toPlace,
            Math.ceil(data.minerSmelterCount / bots.length)
        )
        for(let i=0; i<botMinerSmelter; i++) {
            const x = data.minerSmelterCount - toPlace
            const minerPosition = {x: anchor.x + x * 2, y: anchor.y};
            const furnacePosition = {x: minerPosition.x, y: minerPosition.y + 2};
            const subtask = await createPlaceStarterMinerFurnaceTask(
                store, minerName, furnaceName, minerPosition,
                furnacePosition, data.plateName, data.oreName
            )
            store.commit('addSubTask', {id: task.id, task: subtask})
            queue[playerId].push(subtask)
            toPlace -= 1
        }
    }
    await processBotQueue(store, queue, bots)
    return minerFurnaces
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildStarterMinerFurnaceTask(
    store: Store<State>, oreName: string,
    plateName: string, minerSmelterCount: number
): Promise<Task> {
    const data: TaskData = {
        minerSmelterCount,
        oreName,
        plateName,
    }
    return createTask(TASK_TYPE, `Starter Miner/Furnace for ${oreName} x ${minerSmelterCount}`, data)
}