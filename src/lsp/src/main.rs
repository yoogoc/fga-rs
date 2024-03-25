use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::*;
use tower_lsp::Server;
use tower_lsp::{lsp_types::*, Client, LanguageServer, LspService};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let _ = params;
        let _ = self.client;
        todo!()
    }
    async fn initialized(&self, _: InitializedParams) {
        todo!()
    }

    async fn shutdown(&self) -> Result<()> {
        todo!()
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let _ = params;
        todo!()
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let _ = params;
        todo!()
    }
    async fn will_save(&self, params: WillSaveTextDocumentParams) {
        let _ = params;
    }

    async fn will_save_wait_until(&self, params: WillSaveTextDocumentParams) -> Result<Option<Vec<TextEdit>>> {
        let _ = params;
        todo!()
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let _ = params;
        todo!()
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let _ = params;
        todo!()
    }

    async fn goto_declaration(&self, params: GotoDeclarationParams) -> Result<Option<GotoDeclarationResponse>> {
        let _ = params;
        todo!()
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let _ = params;
        todo!()
    }
    async fn goto_type_definition(
        &self,
        params: GotoTypeDefinitionParams,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let _ = params;
        todo!()
    }
    async fn goto_implementation(
        &self,
        params: GotoImplementationParams,
    ) -> Result<Option<GotoImplementationResponse>> {
        let _ = params;
        todo!()
    }
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let _ = params;
        todo!()
    }
    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        let _ = params;
        todo!()
    }
    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        let _ = params;
        todo!()
    }
    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        let _ = params;
        todo!()
    }
    async fn prepare_type_hierarchy(
        &self,
        params: TypeHierarchyPrepareParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let _ = params;
        todo!()
    }
    async fn supertypes(&self, params: TypeHierarchySupertypesParams) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let _ = params;
        todo!()
    }
    async fn document_highlight(&self, params: DocumentHighlightParams) -> Result<Option<Vec<DocumentHighlight>>> {
        let _ = params;
        todo!()
    }
    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        let _ = params;
        todo!()
    }
    async fn document_link_resolve(&self, params: DocumentLink) -> Result<DocumentLink> {
        let _ = params;
        todo!()
    }
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let _ = params;
        todo!()
    }
    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let _ = params;
        todo!()
    }
    async fn code_lens_resolve(&self, params: CodeLens) -> Result<CodeLens> {
        let _ = params;
        todo!()
    }
    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let _ = params;
        todo!()
    }
    async fn selection_range(&self, params: SelectionRangeParams) -> Result<Option<Vec<SelectionRange>>> {
        let _ = params;
        todo!()
    }
    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        let _ = params;
        todo!()
    }
    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        let _ = params;
        todo!()
    }
    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let _ = params;
        todo!()
    }
    async fn inline_value(&self, params: InlineValueParams) -> Result<Option<Vec<InlineValue>>> {
        let _ = params;
        todo!()
    }
    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let _ = params;
        todo!()
    }
    async fn inlay_hint_resolve(&self, params: InlayHint) -> Result<InlayHint> {
        let _ = params;
        todo!()
    }
    async fn moniker(&self, params: MonikerParams) -> Result<Option<Vec<Moniker>>> {
        let _ = params;
        todo!()
    }
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let _ = params;
        todo!()
    }
    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        let _ = params;
        todo!()
    }
    async fn diagnostic(&self, params: DocumentDiagnosticParams) -> Result<DocumentDiagnosticReportResult> {
        let _ = params;
        todo!()
    }
    async fn workspace_diagnostic(&self, params: WorkspaceDiagnosticParams) -> Result<WorkspaceDiagnosticReportResult> {
        let _ = params;
        todo!()
    }
    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let _ = params;
        todo!()
    }
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let _ = params;
        todo!()
    }
    async fn code_action_resolve(&self, params: CodeAction) -> Result<CodeAction> {
        let _ = params;
        todo!()
    }
    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let _ = params;
        todo!()
    }
    async fn color_presentation(&self, params: ColorPresentationParams) -> Result<Vec<ColorPresentation>> {
        let _ = params;
        todo!()
    }
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let _ = params;
        todo!()
    }
    async fn range_formatting(&self, params: DocumentRangeFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let _ = params;
        todo!()
    }
    async fn on_type_formatting(&self, params: DocumentOnTypeFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let _ = params;
        todo!()
    }
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let _ = params;
        todo!()
    }
    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> Result<Option<PrepareRenameResponse>> {
        let _ = params;
        todo!()
    }
    async fn linked_editing_range(&self, params: LinkedEditingRangeParams) -> Result<Option<LinkedEditingRanges>> {
        let _ = params;
        todo!()
    }
    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        let _ = params;
        todo!()
    }
    async fn symbol_resolve(&self, params: WorkspaceSymbol) -> Result<WorkspaceSymbol> {
        let _ = params;
        todo!()
    }
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let _ = params;
        todo!()
    }
    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let _ = params;
        todo!()
    }
    async fn will_create_files(&self, params: CreateFilesParams) -> Result<Option<WorkspaceEdit>> {
        let _ = params;
        todo!()
    }
    async fn did_create_files(&self, params: CreateFilesParams) {
        let _ = params;
        todo!()
    }
    async fn will_rename_files(&self, params: RenameFilesParams) -> Result<Option<WorkspaceEdit>> {
        let _ = params;
        todo!()
    }
    async fn did_rename_files(&self, params: RenameFilesParams) {
        let _ = params;
        todo!()
    }
    async fn will_delete_files(&self, params: DeleteFilesParams) -> Result<Option<WorkspaceEdit>> {
        let _ = params;
        todo!()
    }
    async fn did_delete_files(&self, params: DeleteFilesParams) {
        let _ = params;
        todo!()
    }
    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let _ = params;
        todo!()
    }
    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        let _ = params;
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
