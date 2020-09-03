import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {countEntitiesFromBlueprint, positionLabel} from "@/factorio-bot/util";
import {
    Direction,
    FactorioBlueprintInfo,
    FactorioBlueprintResult,
    FactorioEntity,
    Position
} from "@/factorio-bot/types";
import {buildBotQueueToCraft, processBotQueue} from "@/factorio-bot/bot-queue";

const TASK_TYPE = 'build-blueprint'

type TaskData = {
    blueprint: FactorioBlueprintInfo,
    position: Position,
    direction: Direction,
    immediate: boolean,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity[]> {
    const data: TaskData = task.data as TaskData
    const firstBot = bots[0]
    if (!data.immediate) {
        const container = data.blueprint.data as FactorioBlueprintResult
        const entities = countEntitiesFromBlueprint(container.blueprint)
        // each bot should first craft what it needs
        const queue = await buildBotQueueToCraft(store, task, bots, entities)
        await processBotQueue(store, queue, bots)
    }
    return await firstBot.placeBlueprint(
        data.blueprint.blueprint,
        data.position,
        data.direction,
        false,
        bots.slice(1).map(bot => bot.playerId)
    );
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildBlueprintTask(store: Store<State>, blueprint: FactorioBlueprintInfo,
                                               position: Position,
                                               direction: Direction,
                                               immediate: boolean): Promise<Task> {
    const data: TaskData = {
        blueprint,
        position,
        direction,
        immediate,
    }
    return createTask(TASK_TYPE, `Build Blueprint '${blueprint.label}' @ ${positionLabel(position)} (${direction})`, data)
}